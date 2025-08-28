import { expect, Page } from "@playwright/test";

export interface CreateUserResult {
  userFullName: string;
  userEmail: string;
  userPassword: string;
}

export async function createUserWrapper(page: Page): Promise<CreateUserResult> {
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

  return {
    userFullName: USER_FULL_NAME,
    userEmail: USER_EMAIL,
    userPassword: USER_PASSWORD,
  };
}

export interface UserLoginData {
  userEmail: string;
  userPassword: string;
}

export async function loginUserWrapper(
  page: Page,
  loginData: UserLoginData | CreateUserResult
) {
  await page.goto("http://localhost:3000/");

  await page
    .locator("#navbarSupportedContent")
    .getByRole("link", { name: "Login" })
    .click();

  await page
    .getByRole("textbox", { name: "Email address" })
    .fill(loginData.userEmail);

  await page
    .getByRole("textbox", { name: "Password" })
    .fill(loginData.userPassword);

  await page.getByRole("button", { name: "Sign in" }).click();

  await page.waitForLoadState("domcontentloaded");
}
