import { expect, test } from "@playwright/test";
import { createUserWrapper, loginUserWrapper } from "../fixtures/user";

let CREATED_POST_URL: string;

let UPDATE_USER_NAME = "userupdatedname";

test.describe("Create Post and Comment Test", () => {
  test.describe.configure({
    mode: "serial",
  });

  test.beforeAll(async ({ browser }) => {
    let page = await browser.newPage();
    await createUserWrapper(page);
  });

  test("update user name", async ({ page }) => {
    await page.goto("http://localhost:3000/");

    await loginUserWrapper(page);

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

    await loginUserWrapper(page);

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
});
