let USER_FULL_NAME;
let USER_EMAIL;
let USER_PASSWORD;

export function getUserFullName() {
  if (!USER_FULL_NAME) {
    USER_FULL_NAME = `usertest${Date.now()}`;
  }

  return USER_FULL_NAME;
}

export function getUserEmail() {
  if (!USER_EMAIL) {
    USER_EMAIL = `${getUserFullName()}@example.com`;
  }

  return USER_EMAIL;
}

export function getUserPassword() {
  if (!USER_PASSWORD) {
    USER_PASSWORD = `${getUserFullName()}-password`;
  }

  return USER_PASSWORD;
}
