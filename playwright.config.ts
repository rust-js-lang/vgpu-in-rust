import {defineConfig} from '@playwright/test';
import {availableParallelism} from 'node:os';
export default defineConfig({
  testDir:'tests/browser',fullyParallel:true,workers:availableParallelism(),
  forbidOnly:!!process.env.CI,retries:0,timeout:15000,
  expect:{timeout:5000},reporter:'list',
  use:{baseURL:'http://127.0.0.1:4193',trace:'retain-on-failure',launchOptions:{args:['--enable-unsafe-webgpu']}},
  webServer:{command:'bunx --bun vite build --config tests/browser/vite.config.ts && bunx --bun vite preview --config tests/browser/vite.config.ts --host 127.0.0.1 --port 4193 --strictPort',url:'http://127.0.0.1:4193',reuseExistingServer:false,timeout:30000},
});
