import { expect, test } from "vitest";
import {connectTargets, filterConnectedTargets, loadTargets} from "../src";

test("Ping nodes", async () => {
  const targets = loadTargets();

  const attemptedConnections = await connectTargets(targets);
  const connectedTargets = filterConnectedTargets(attemptedConnections);

  // TODO Make this test check a minimum number of connected targets once we have a list of HPs to use.
  expect(connectedTargets.length).equal(targets.length, 'All targets should have been connected to');
});
