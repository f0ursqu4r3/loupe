/**
 * Query Execution Load Test
 *
 * Tests query creation and execution performance under concurrent load
 * Tests connection pool behavior and query limiter
 *
 * Usage:
 *   k6 run query-execution.js
 */

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';
import { SharedArray } from 'k6/data';

// Custom metrics
const queryCreationDuration = new Trend('query_creation_duration');
const runCreationDuration = new Trend('run_creation_duration');
const runCompletionDuration = new Trend('run_completion_duration');
const concurrentRunLimit = new Counter('concurrent_run_limit_hits');
const queryErrors = new Counter('query_errors');
const successRate = new Rate('success_rate');

export const options = {
  scenarios: {
    // Concurrent query execution test
    concurrent_queries: {
      executor: 'constant-arrival-rate',
      rate: 50, // 50 iterations per second
      timeUnit: '1s',
      duration: '2m',
      preAllocatedVUs: 100,
      maxVUs: 200,
      exec: 'concurrentQueryExecution',
    },

    // Stress test - exceed query limits
    stress_test: {
      executor: 'ramping-arrival-rate',
      startRate: 10,
      timeUnit: '1s',
      stages: [
        { duration: '30s', target: 50 },  // Ramp to 50/s
        { duration: '1m', target: 100 },   // Ramp to 100/s
        { duration: '30s', target: 200 },  // Spike to 200/s
        { duration: '30s', target: 0 },    // Ramp down
      ],
      preAllocatedVUs: 200,
      maxVUs: 500,
      exec: 'stressTestExecution',
      startTime: '2m30s', // Start after concurrent test
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<5000', 'p(99)<10000'],
    'http_req_failed': ['rate<0.10'], // 10% tolerance for stress test
    'query_creation_duration': ['p(95)<2000'],
    'run_creation_duration': ['p(95)<1000'],
    'run_completion_duration': ['p(95)<3000'],
    'success_rate': ['rate>0.80'], // 80% success during stress
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API_BASE = `${BASE_URL}/api/v1`;

// Sample SQL queries for testing
const SQL_QUERIES = new SharedArray('sql_queries', function () {
  return [
    'SELECT COUNT(*) FROM users',
    'SELECT * FROM users LIMIT 100',
    'SELECT date, COUNT(*) FROM events GROUP BY date ORDER BY date DESC LIMIT 30',
    'SELECT user_id, SUM(amount) as total FROM transactions GROUP BY user_id ORDER BY total DESC LIMIT 10',
    'SELECT * FROM products WHERE category = $1 ORDER BY price',
    'SELECT AVG(score) as avg_score, MIN(score) as min_score, MAX(score) as max_score FROM metrics',
  ];
});

let testData = {};

export function setup() {
  console.log('Setting up query execution test...');

  // Create test user
  const user = {
    email: `query-loadtest-${Date.now()}@example.com`,
    password: 'LoadTest123!',
    name: 'Query Load Test User',
  };

  const registerResponse = http.post(
    `${API_BASE}/auth/register`,
    JSON.stringify(user),
    { headers: { 'Content-Type': 'application/json' } }
  );

  if (registerResponse.status !== 201) {
    throw new Error('Setup failed: Could not create user');
  }

  const userData = JSON.parse(registerResponse.body);

  // Login
  const loginResponse = http.post(
    `${API_BASE}/auth/login`,
    JSON.stringify({ email: user.email, password: user.password }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  const loginData = JSON.parse(loginResponse.body);
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${loginData.token}`,
  };

  // Create datasource
  const datasource = {
    org_id: userData.org_id,
    user_id: userData.id,
    name: 'Load Test Datasource',
    ds_type: 'postgres',
    connection_string: 'postgres://localhost:5432/test',
  };

  const dsResponse = http.post(
    `${API_BASE}/datasources`,
    JSON.stringify(datasource),
    { headers }
  );

  if (dsResponse.status !== 201) {
    throw new Error('Setup failed: Could not create datasource');
  }

  const dsData = JSON.parse(dsResponse.body);

  testData = {
    userId: userData.id,
    orgId: userData.org_id,
    datasourceId: dsData.id,
    token: loginData.token,
  };

  console.log(`Setup complete. Datasource: ${testData.datasourceId}`);
  return testData;
}

export function concurrentQueryExecution(data) {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${data.token}`,
  };

  group('Query Execution Flow', () => {
    // Pick random SQL query
    const sql = SQL_QUERIES[Math.floor(Math.random() * SQL_QUERIES.length)];

    // Create query
    const query = {
      org_id: data.orgId,
      user_id: data.userId,
      datasource_id: data.datasourceId,
      name: `Load Test Query ${__VU}-${__ITER}`,
      description: 'Performance test query',
      sql: sql,
      parameters: [],
      timeout_seconds: 30,
      max_rows: 1000,
    };

    const createStart = Date.now();
    const createResponse = http.post(
      `${API_BASE}/queries`,
      JSON.stringify(query),
      { headers, tags: { name: 'CreateQuery' } }
    );
    queryCreationDuration.add(Date.now() - createStart);

    if (createResponse.status === 201) {
      const createdQuery = JSON.parse(createResponse.body);

      // Create run
      const run = {
        org_id: data.orgId,
        query_id: createdQuery.id,
        bound_params: {},
      };

      const runStart = Date.now();
      const runResponse = http.post(
        `${API_BASE}/runs`,
        JSON.stringify(run),
        { headers, tags: { name: 'CreateRun' } }
      );
      runCreationDuration.add(Date.now() - runStart);

      const runSuccess = check(runResponse, {
        'run created successfully': (r) => r.status === 201 || r.status === 429,
      });

      if (runResponse.status === 429) {
        // Hit concurrent query limit
        concurrentRunLimit.add(1);
      } else if (runResponse.status === 201) {
        successRate.add(1);
      } else {
        queryErrors.add(1);
        successRate.add(0);
      }
    } else {
      queryErrors.add(1);
      successRate.add(0);
    }
  });

  sleep(0.1); // Brief pause
}

export function stressTestExecution(data) {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${data.token}`,
  };

  // Rapid-fire query creation
  const query = {
    org_id: data.orgId,
    user_id: data.userId,
    datasource_id: data.datasourceId,
    name: `Stress Test ${__VU}-${__ITER}`,
    sql: 'SELECT 1',
    parameters: [],
    timeout_seconds: 10,
    max_rows: 10,
  };

  const response = http.post(
    `${API_BASE}/queries`,
    JSON.stringify(query),
    { headers, tags: { name: 'StressCreateQuery' } }
  );

  if (response.status === 201) {
    const createdQuery = JSON.parse(response.body);

    // Immediate run creation (stress the queue)
    const run = {
      org_id: data.orgId,
      query_id: createdQuery.id,
      bound_params: {},
    };

    const runResponse = http.post(
      `${API_BASE}/runs`,
      JSON.stringify(run),
      { headers, tags: { name: 'StressCreateRun' } }
    );

    if (runResponse.status === 429) {
      concurrentRunLimit.add(1);
    }
  }
}

export function teardown(data) {
  console.log('Query execution load test completed');
  console.log(`Datasource ID: ${data.datasourceId}`);
}
