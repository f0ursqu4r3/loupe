/**
 * Dashboard API Load Test
 *
 * Tests dashboard listing and retrieval performance under load
 *
 * Usage:
 *   k6 run dashboard-api.js
 *   k6 run --vus 100 --duration 60s dashboard-api.js
 */

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

// Custom metrics
const listDashboardsDuration = new Trend('list_dashboards_duration');
const getDashboardDuration = new Trend('get_dashboard_duration');
const createDashboardDuration = new Trend('create_dashboard_duration');
const cacheHitRate = new Rate('cache_hit_rate');
const apiErrors = new Counter('api_errors');

export const options = {
  scenarios: {
    // Read-heavy workload (80% reads, 20% writes)
    read_heavy: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 50 },
        { duration: '2m', target: 100 },
        { duration: '30s', target: 0 },
      ],
      exec: 'readHeavyScenario',
    },

    // Write-heavy workload (creates and updates)
    write_heavy: {
      executor: 'constant-vus',
      vus: 20,
      duration: '3m',
      exec: 'writeHeavyScenario',
      startTime: '10s',
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<1500', 'p(99)<3000'],
    'http_req_failed': ['rate<0.05'], // 5% error tolerance during load
    'list_dashboards_duration': ['p(95)<800'],
    'get_dashboard_duration': ['p(95)<500'],
    'create_dashboard_duration': ['p(95)<2000'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API_BASE = `${BASE_URL}/api/v1`;

// Shared test data (created in setup)
let testData = {};

export function setup() {
  console.log('Setting up test data...');

  // Register a test user
  const user = {
    email: `dashboard-loadtest-${Date.now()}@example.com`,
    password: 'LoadTest123!',
    name: 'Dashboard Load Test User',
  };

  const registerResponse = http.post(
    `${API_BASE}/auth/register`,
    JSON.stringify(user),
    { headers: { 'Content-Type': 'application/json' } }
  );

  if (registerResponse.status !== 201) {
    throw new Error(`Setup failed: Could not create user (${registerResponse.status})`);
  }

  const userData = JSON.parse(registerResponse.body);

  // Login to get token
  const loginResponse = http.post(
    `${API_BASE}/auth/login`,
    JSON.stringify({ email: user.email, password: user.password }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  if (loginResponse.status !== 200) {
    throw new Error('Setup failed: Could not login');
  }

  const loginData = JSON.parse(loginResponse.body);

  testData = {
    userId: userData.id,
    orgId: userData.org_id,
    token: loginData.token,
    email: user.email,
  };

  console.log(`Setup complete. User: ${testData.email}, Org: ${testData.orgId}`);
  return testData;
}

export function readHeavyScenario(data) {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${data.token}`,
  };

  // 80% of requests are reads
  const readWeight = Math.random();

  if (readWeight < 0.8) {
    // List dashboards
    group('List Dashboards', () => {
      const start = Date.now();
      const response = http.get(
        `${API_BASE}/dashboards?org_id=${data.orgId}&limit=20&offset=0`,
        { headers, tags: { name: 'ListDashboards' } }
      );
      listDashboardsDuration.add(Date.now() - start);

      const success = check(response, {
        'list dashboards status is 200': (r) => r.status === 200,
        'list has items array': (r) => Array.isArray(JSON.parse(r.body).items),
      });

      if (!success) apiErrors.add(1);

      // Check for cache hit (custom header from server)
      if (response.headers['X-Cache-Hit']) {
        cacheHitRate.add(1);
      } else {
        cacheHitRate.add(0);
      }
    });
  } else {
    // Create dashboard (20% of requests)
    group('Create Dashboard', () => {
      const dashboard = {
        org_id: data.orgId,
        user_id: data.userId,
        name: `Load Test Dashboard ${__VU}-${__ITER}`,
        description: 'Created during load test',
        layout_config: {},
        tags: ['loadtest', 'performance'],
      };

      const start = Date.now();
      const response = http.post(
        `${API_BASE}/dashboards`,
        JSON.stringify(dashboard),
        { headers, tags: { name: 'CreateDashboard' } }
      );
      createDashboardDuration.add(Date.now() - start);

      const success = check(response, {
        'create dashboard status is 201': (r) => r.status === 201,
        'created dashboard has id': (r) => JSON.parse(r.body).id !== undefined,
      });

      if (!success) apiErrors.add(1);
    });
  }

  sleep(Math.random() * 2 + 0.5); // Random sleep 0.5-2.5s
}

export function writeHeavyScenario(data) {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${data.token}`,
  };

  group('Create and Update Dashboard', () => {
    // Create
    const dashboard = {
      org_id: data.orgId,
      user_id: data.userId,
      name: `Write Test Dashboard ${__VU}-${__ITER}`,
      description: 'Heavy write test',
      layout_config: { columns: 12 },
      tags: ['write-test'],
    };

    const createStart = Date.now();
    const createResponse = http.post(
      `${API_BASE}/dashboards`,
      JSON.stringify(dashboard),
      { headers, tags: { name: 'CreateDashboard' } }
    );
    createDashboardDuration.add(Date.now() - createStart);

    if (createResponse.status === 201) {
      const createdDashboard = JSON.parse(createResponse.body);

      sleep(0.5);

      // Update
      const updatePayload = {
        name: `Updated ${dashboard.name}`,
        description: 'Updated during load test',
      };

      const updateResponse = http.put(
        `${API_BASE}/dashboards/${createdDashboard.id}?org_id=${data.orgId}`,
        JSON.stringify(updatePayload),
        { headers, tags: { name: 'UpdateDashboard' } }
      );

      check(updateResponse, {
        'update dashboard status is 200': (r) => r.status === 200,
      });

      sleep(0.5);

      // Delete
      const deleteResponse = http.del(
        `${API_BASE}/dashboards/${createdDashboard.id}?org_id=${data.orgId}`,
        { headers, tags: { name: 'DeleteDashboard' } }
      );

      check(deleteResponse, {
        'delete dashboard status is 204': (r) => r.status === 204,
      });
    } else {
      apiErrors.add(1);
    }
  });

  sleep(1);
}

export function teardown(data) {
  console.log('Dashboard load test completed');
  console.log(`Test user: ${data.email}`);
}
