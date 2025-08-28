import { expect, test } from "@playwright/test";
import {
  getUserEmail,
  getUserFullName,
  getUserPassword,
} from "../constants/user";

test.describe("User Login And Register Test", () => {
  test.describe.configure({
    mode: "serial",
  });

  test("register", async ({ page }) => {
    await page.goto("http://localhost:3000");

    await page.click(
      "#navbarSupportedContent > ul.navbar-nav.mr-auto.mb-2.mb-lg-0 > li:nth-child(1) > a"
    );

    await page.fill("#inputName", getUserFullName());

    await page.fill("#inputEmail", getUserEmail());

    await page.fill("#inputPassword", getUserPassword());

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
    await page.goto("http://localhost:3000/");
    await page.getByRole("link", { name: "Login" }).click();

    await page
      .getByRole("textbox", { name: "Email address" })
      .fill(getUserEmail());

    await page
      .getByRole("textbox", { name: "Password" })
      .fill(getUserPassword());

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
