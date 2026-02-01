/**
 * Connection Pool Stress Test
 *
 * Validates database connection pool behavior under extreme load
 * Tests pool exhaustion, acquisition timeout, connection reuse
 *
 * Usage:
 *   k6 run connection-pool-stress.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

// Custom metrics
const poolAcquisitionErrors = new Counter('pool_acquisition_errors');
const poolTimeouts = new Counter('pool_timeouts');
const connectionReuse = new Rate('connection_reuse');
const dbOperationDuration = new Trend('db_operation_duration');

export const options = {
  scenarios: {
    // Gradual ramp to test pool scaling
    pool_ramp: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 20 },  // Below pool max
        { duration: '30s', target: 50 },  // At pool max
        { duration: '1m', target: 100 },  // Above pool max (queuing)
        { duration: '1m', target: 200 },  // 2x pool max (heavy queuing)
        { duration: '30s', target: 0 },   // Ramp down
      ],
      exec: 'testPoolBehavior',
    },

    // Burst test - sudden spike
    pool_burst: {
      executor: 'constant-vus',
      vus: 300,
      duration: '30s',
      exec: 'testPoolExhaustion',
      startTime: '4m', // Start after ramp test
    },

    // Long-running queries (hold connections)
    long_running: {
      executor: 'constant-vus',
      vus: 10,
      duration: '5m',
      exec: 'testLongRunningQueries',
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<10000'], // Tolerate slower under stress
    'pool_acquisition_errors': ['count<100'], // Max 100 acquisition errors
    'pool_timeouts': ['count<50'], // Max 50 timeouts
    'db_operation_duration': ['p(95)<5000'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API_BASE = `${BASE_URL}/api/v1`;
const METRICS_URL = `${BASE_URL}/metrics`;

let testData = {};

export function setup() {
  console.log('Setting up connection pool stress test...');

  // Create test user
  const user = {
    email: `pool-test-${Date.now()}@example.com`,
    password: 'PoolTest123!',
    name: 'Pool Stress Test User',
  };

  const registerResponse = http.post(
    `${API_BASE}/auth/register`,
    JSON.stringify(user),
    { headers: { 'Content-Type': 'application/json' } }
  );

  const userData = JSON.parse(registerResponse.body);

  const loginResponse = http.post(
    `${API_BASE}/auth/login`,
    JSON.stringify({ email: user.email, password: user.password }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  const loginData = JSON.parse(loginResponse.body);

  testData = {
    userId: userData.id,
    orgId: userData.org_id,
    token: loginData.token,
  };

  // Get initial pool stats
  const metricsResponse = http.get(METRICS_URL);
  console.log('Initial pool state captured');

  return testData;
}

export function testPoolBehavior(data) {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${data.token}`,
  };

  // Perform database-heavy operations
  const start = Date.now();

  // List dashboards (requires DB connection)
  const listResponse = http.get(
    `${API_BASE}/dashboards?org_id=${data.orgId}&limit=50`,
    { headers, tags: { name: 'ListDashboards' } }
  );

  const duration = Date.now() - start;
  dbOperationDuration.add(duration);

  const success = check(listResponse, {
    'list dashboards success': (r) => r.status === 200,
    'no pool timeout': (r) => !r.body.includes('pool timeout'),
    'no connection error': (r) => !r.body.includes('connection'),
  });

  if (!success) {
    if (listResponse.body.includes('timeout')) {
      poolTimeouts.add(1);
    }
    if (listResponse.body.includes('acquisition') || listResponse.body.includes('pool')) {
      poolAcquisitionErrors.add(1);
    }
  }

  // Check connection reuse via metrics
  if (__ITER % 10 === 0) {
    const metricsResponse = http.get(METRICS_URL, { tags: { name: 'Metrics' } });
    if (metricsResponse.status === 200) {
      // Parse Prometheus metrics for pool stats
      const body = metricsResponse.body;
      if (body.includes('db_pool_connections_idle')) {
        connectionReuse.add(1);
      } else {
        connectionReuse.add(0);
      }
    }
  }

  sleep(0.1);
}

export function testPoolExhaustion(data) {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${data.token}`,
  };

  // Rapid-fire requests to exhaust pool
  const responses = http.batch([
    ['GET', `${API_BASE}/dashboards?org_id=${data.orgId}`, null, { headers }],
    ['GET', `${API_BASE}/queries?org_id=${data.orgId}`, null, { headers }],
    ['GET', `${API_BASE}/datasources?org_id=${data.orgId}`, null, { headers }],
    ['GET', `${API_BASE}/visualizations?org_id=${data.orgId}`, null, { headers }],
  ]);

  responses.forEach((response) => {
    if (response.status !== 200) {
      if (response.body && response.body.includes('pool')) {
        poolAcquisitionErrors.add(1);
      }
    }
  });
}

export function testLongRunningQueries(data) {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${data.token}`,
  };

  // Simulate long-running operations that hold connections
  const response = http.get(
    `${API_BASE}/dashboards?org_id=${data.orgId}&limit=100&offset=0`,
    {
      headers,
      timeout: '30s', // Allow up to 30s
      tags: { name: 'LongRunning' }
    }
  );

  check(response, {
    'long query completed': (r) => r.status === 200 || r.status === 504,
  });

  sleep(5); // Hold for a bit before next iteration
}

export function teardown(data) {
  console.log('Fetching final pool statistics...');

  // Get final pool metrics
  const metricsResponse = http.get(METRICS_URL);

  if (metricsResponse.status === 200) {
    const body = metricsResponse.body;

    // Extract key metrics
    console.log('=== Final Pool Statistics ===');

    // Parse active connections
    const activeMatch = body.match(/db_pool_connections_active\s+(\d+)/);
    if (activeMatch) {
      console.log(`Active connections: ${activeMatch[1]}`);
    }

    // Parse idle connections
    const idleMatch = body.match(/db_pool_connections_idle\s+(\d+)/);
    if (idleMatch) {
      console.log(`Idle connections: ${idleMatch[1]}`);
    }

    // Parse max connections
    const maxMatch = body.match(/db_pool_connections_max\s+(\d+)/);
    if (maxMatch) {
      console.log(`Max connections: ${maxMatch[1]}`);
    }

    // Parse acquisition timeouts
    const timeoutMatch = body.match(/db_pool_acquisition_timeouts_total\s+(\d+)/);
    if (timeoutMatch) {
      console.log(`Total acquisition timeouts: ${timeoutMatch[1]}`);
    }
  }

  console.log('Connection pool stress test completed');
}
