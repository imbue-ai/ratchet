// TypeScript file with multi-line content
// TODO: refactor this module

interface User {
  name: string;
  email: string;
}

function processUser(user: User): void {
  // FIXME: validate input
  console.log(user.name);
}

// TODO: add more types
type Status = "active" | "inactive";

class Service {
  // FIXME: implement properly
  async fetch(): Promise<void> {
    // TODO: add error handling
    return Promise.resolve();
  }
}
