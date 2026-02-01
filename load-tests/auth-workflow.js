/**
 * Authentication Workflow Load Test
 *
 * Tests login/registration performance under load
 *
 * Usage:
 *   k6 run auth-workflow.js
 *   k6 run --vus 50 --duration 30s auth-workflow.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

// Custom metrics
const registrationDuration = new Trend('registration_duration');
const loginDuration = new Trend('login_duration');
const registrationErrors = new Counter('registration_errors');
const loginErrors = new Counter('login_errors');
const successRate = new Rate('success_rate');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 10 },  // Ramp-up to 10 users
    { duration: '1m', target: 50 },   // Ramp-up to 50 users
    { duration: '2m', target: 50 },   // Stay at 50 users
    { duration: '30s', target: 100 }, // Spike to 100 users
    { duration: '1m', target: 100 },  // Stay at 100 users
    { duration: '30s', target: 0 },   // Ramp-down to 0 users
  ],
  thresholds: {
    'http_req_duration': ['p(95)<2000', 'p(99)<5000'], // 95% < 2s, 99% < 5s
    'http_req_failed': ['rate<0.01'],                   // Error rate < 1%
    'success_rate': ['rate>0.95'],                      // Success rate > 95%
    'registration_duration': ['p(95)<3000'],            // Registration p95 < 3s
    'login_duration': ['p(95)<1000'],                   // Login p95 < 1s
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API_BASE = `${BASE_URL}/api/v1`;

// Generate unique user data
function generateUser(vuId, iterationId) {
  return {
    email: `loadtest-${vuId}-${iterationId}-${Date.now()}@example.com`,
    password: 'LoadTest123!@#',
    name: `Load Test User ${vuId}-${iterationId}`,
  };
}

export default function () {
  const user = generateUser(__VU, __ITER);

  // Test 1: User Registration
  const registerPayload = JSON.stringify(user);
  const registerParams = {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { name: 'RegisterUser' },
  };

  const registerStart = Date.now();
  const registerResponse = http.post(
    `${API_BASE}/auth/register`,
    registerPayload,
    registerParams
  );
  const registerEnd = Date.now();

  const registerSuccess = check(registerResponse, {
    'registration status is 201': (r) => r.status === 201,
    'registration has user id': (r) => JSON.parse(r.body).id !== undefined,
    'registration has email': (r) => JSON.parse(r.body).email === user.email,
    'registration has org_id': (r) => JSON.parse(r.body).org_id !== undefined,
  });

  registrationDuration.add(registerEnd - registerStart);
  if (!registerSuccess) {
    registrationErrors.add(1);
  }
  successRate.add(registerSuccess);

  sleep(0.5); // Brief pause between registration and login

  // Test 2: User Login
  const loginPayload = JSON.stringify({
    email: user.email,
    password: user.password,
  });
  const loginParams = {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { name: 'LoginUser' },
  };

  const loginStart = Date.now();
  const loginResponse = http.post(
    `${API_BASE}/auth/login`,
    loginPayload,
    loginParams
  );
  const loginEnd = Date.now();

  const loginSuccess = check(loginResponse, {
    'login status is 200': (r) => r.status === 200,
    'login has token': (r) => JSON.parse(r.body).token !== undefined,
    'login has user': (r) => JSON.parse(r.body).user !== undefined,
  });

  loginDuration.add(loginEnd - loginStart);
  if (!loginSuccess) {
    loginErrors.add(1);
  }
  successRate.add(loginSuccess);

  sleep(1); // Pause before next iteration
}

// Setup function (runs once before all VUs)
export function setup() {
  console.log(`Starting auth workflow load test against ${API_BASE}`);
  console.log('Test stages:');
  console.log('  1. Ramp-up to 10 users (30s)');
  console.log('  2. Ramp-up to 50 users (1m)');
  console.log('  3. Sustain 50 users (2m)');
  console.log('  4. Spike to 100 users (30s)');
  console.log('  5. Sustain 100 users (1m)');
  console.log('  6. Ramp-down (30s)');
}

// Teardown function (runs once after all VUs)
export function teardown(data) {
  console.log('Auth workflow load test completed');
}
