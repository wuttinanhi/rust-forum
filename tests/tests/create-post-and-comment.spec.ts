import { expect, test } from "@playwright/test";
import {
  CreateUserResult,
  createUserWrapper,
  loginUserWrapper,
} from "../fixtures/user";

let CREATED_POST_URL: string;

test.describe("Create Post and Comment Test", () => {
  let CREATED_USER: CreateUserResult;

  test.describe.configure({
    mode: "serial",
  });

  test.beforeEach(async ({ browser }) => {
    let page = await browser.newPage();

    CREATED_USER = await createUserWrapper(page);
  });

  test("create post", async ({ page }) => {
    await page.goto("http://localhost:3000/");

    await loginUserWrapper(page, CREATED_USER);

    await page.waitForLoadState("domcontentloaded");

    await page.click("#post_menu > a");

    await expect(page.getByRole("heading")).toContainText("Create new post");

    await page.getByRole("textbox", { name: "Post title" }).fill("test post 1");
    await page
      .getByRole("textbox", { name: "post body" })
      .fill("test post 1 body");

    await page.getByRole("button", { name: "Create" }).click();

    await expect(page.getByRole("alert")).toBeVisible();
    await expect(page.getByRole("alert")).toContainText("Created post!");
    await expect(page.locator("body")).toContainText("test post 1");
    await expect(page.locator("body")).toContainText("test post 1 body");

    CREATED_POST_URL = page.url();
  });

  test("create comment", async ({ page }) => {
    await page.goto("http://localhost:3000/");

    await loginUserWrapper(page, CREATED_USER);

    await page.goto(CREATED_POST_URL);

    await page.waitForLoadState("domcontentloaded");

    await page.getByPlaceholder("New comment").fill("comment 1");

    await page.getByRole("button", { name: "Comment" }).click();

    await page.waitForLoadState("domcontentloaded");

    await expect(
      page.locator('//*[@id="comments"]/div[1]/div[1]/p')
    ).toContainText("comment 1");
  });
});
