import { expect, test } from "@playwright/test";
import {
  CreateUserResult,
  createUserWrapper,
  loginUserWrapper,
} from "../fixtures/user";

test.describe("Create Post and Comment Test", () => {
  let UPDATE_USER_NAME = "userupdatedname";
  let NEW_USER_PASSWORD = "user-password-updated";

  let CREATED_USER: CreateUserResult;

  test.describe.configure({
    mode: "serial",
  });

  test.beforeEach(async ({ browser }) => {
    let page = await browser.newPage();

    CREATED_USER = await createUserWrapper(page);
  });

  test("update user name", async ({ page }) => {
    await page.goto("http://localhost:3000/");

    await loginUserWrapper(page, CREATED_USER);

    await page.goto("http://localhost:3000/users/settings");

    await page.getByRole("textbox", { name: "Name" }).fill(UPDATE_USER_NAME);

    await page.getByRole("button", { name: "Save" }).click();

    await page.waitForLoadState("domcontentloaded");

    await expect(page.locator("#notification > div > p")).toContainText(
      "Updated user data"
    );

    await expect(
      page.locator(
        "body > div > div.row.my-5 > div.col-6.mb-5 > form:nth-child(3) > div.my-2 > h3"
      )
    ).toHaveText(UPDATE_USER_NAME);
  });

  test("update user profile picture", async ({ page }) => {
    await page.goto("http://localhost:3000/");

    await loginUserWrapper(page, CREATED_USER);

    await page.goto("http://localhost:3000/users/settings");

    const beforeUpdateProfileImageSrc = await page
      .locator(
        "body > div > div.row.my-5 > div.col-6.mb-5 > form:nth-child(3) > div.my-2.flex.flex-col.justify-center > img"
      )
      .getAttribute("src");

    // await page.getByRole("button", { name: "Profile picture" }).click();
    await page
      .getByRole("button", { name: "Profile picture" })
      .setInputFiles("test.jpg");

    await page.getByRole("button", { name: "Upload" }).click();

    await page.waitForLoadState("domcontentloaded");

    await expect(page.getByRole("img")).toBeVisible();

    await expect(page.getByRole("paragraph")).toContainText(
      "Profile picture uploaded"
    );

    const afterUpdateProfileImageSrc = await page
      .locator(
        "body > div > div.row.my-5 > div.col-6.mb-5 > form:nth-child(3) > div.my-2.flex.flex-col.justify-center > img"
      )
      .getAttribute("src");

    expect(beforeUpdateProfileImageSrc).not.toEqual(afterUpdateProfileImageSrc);
  });

  test("update user password", async ({ page }) => {
    await page.goto("http://localhost:3000/");

    await loginUserWrapper(page, CREATED_USER);

    await page.goto("http://localhost:3000/users/settings");

    await page.locator("#current_password").fill(CREATED_USER.userPassword);

    await page.locator("#new_password").fill(NEW_USER_PASSWORD);

    await page.locator("#confirm_password").fill(NEW_USER_PASSWORD);

    await page.click("#submit-change-password");

    await page.waitForLoadState("domcontentloaded");

    await expect(page.locator("#notification > div > p")).toContainText(
      "Change user password completed!"
    );

    // Logout!
    await page.click("#btn-logout");

    // Login Again Using new password
    await page.goto("http://localhost:3000/users/login");

    await page
      .getByRole("textbox", { name: "Email address" })
      .fill(CREATED_USER.userEmail);

    await page
      .getByRole("textbox", { name: "Password" })
      .fill(NEW_USER_PASSWORD);

    await page.getByRole("button", { name: "Sign in" }).click();

    await page.waitForLoadState("domcontentloaded");

    // navbar should have link menu Posts and User After Login
    await expect(
      page.locator(
        "#navbarSupportedContent > ul.navbar-nav.mr-auto.mb-2.mb-lg-0 > li:nth-child(1) > a"
      )
    ).toBeVisible();

    await expect(
      page.locator(
        "#navbarSupportedContent > ul.navbar-nav.mr-auto.mb-2.mb-lg-0 > li:nth-child(2) > a"
      )
    ).toBeVisible();
  });
});
