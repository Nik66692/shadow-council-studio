import { expect, it } from "vitest";
import { sprint0Sections } from "./index";
it("lists the Sprint 0 navigation sections", () =>
  expect(sprint0Sections).toContain("Dashboard"));
