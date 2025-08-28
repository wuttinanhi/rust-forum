import { expect, test } from "@playwright/test";
import { createUserWrapper } from "../fixtures/user";

test.describe("User Login And Register Test", () => {
  test.describe.configure({
    mode: "serial",
  });

  test("register", async ({ page }) => {
    let USER_FULL_NAME = `usertest${Date.now()}`;
    let USER_EMAIL = `${USER_FULL_NAME}@example.com`;
    let USER_PASSWORD = `${USER_FULL_NAME}-password`;

    await page.goto("http://localhost:3000");

    await page.click(
      "#navbarSupportedContent > ul.navbar-nav.mr-auto.mb-2.mb-lg-0 > li:nth-child(1) > a"
    );

    await page.fill("#inputName", USER_FULL_NAME);

    await page.fill("#inputEmail", USER_EMAIL);

    await page.fill("#inputPassword", USER_PASSWORD);

    await page.click(
      "body > div > div.row.mt-5 > div.col-6 > form > div > label > input[type=checkbox]"
    );

    await page.click("body > div > div.row.mt-5 > div.col-6 > form > button");

    //*[@id="notification"]
    await expect(page.locator("#notification > div > p")).toHaveText(
      "Created user. you can now login!"
    );
  });

  test("login", async ({ page }) => {
    let CREATED_USER = await createUserWrapper(page);

    await page.goto("http://localhost:3000/");
    await page.getByRole("link", { name: "Login" }).click();

    await page
      .getByRole("textbox", { name: "Email address" })
      .fill(CREATED_USER.userEmail);

    await page
      .getByRole("textbox", { name: "Password" })
      .fill(CREATED_USER.userPassword);

    await page.getByRole("button", { name: "Sign in" }).click();

    // navbar should have link menu Posts and User
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
