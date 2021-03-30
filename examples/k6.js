import http from 'k6/http';
import { sleep } from 'k6';

export let options = {
  executor: 'ramping-vus',
  startVUs: 0,
  stages: [
    { duration: '10s', target: 5 },
    { duration: '1m', target: 5 },
    { duration: '10s', target: 0 },
  ],

  // discardResponseBodies: true,
  noConnectionReuse: true,
  noVUConnectionReuse: true,
  batchPerHost: 1,
  userAgent: 'k6',

  noUsageReport: true,
};

const params = {
  headers: {
    'custom-header': 'hello there',
  }
}

export default () => {
  http.get('http://localhost:4000/', params);
  sleep(0.1) // seconds
}
