import { test, expect } from '@playwright/test';

test.describe.only('focused group', () => {
    test('inside focused group', async ({ page }) => {
        await expect(page).toHaveTitle(/app/);
    });
});

test.only('standalone focused test', async ({ page }) => {
    await expect(page).toHaveTitle(/app/);
});

test('normal test without only', async ({ page }) => {
    await expect(page).toHaveTitle(/app/);
});
