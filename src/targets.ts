import fs from 'fs';
import path from 'path';
import {TryCpClient} from "@holochain/tryorama";

/**
 * Load the targets.json file and return the nodes field.
 */
export const loadTargets = (): string[] => {
    const targetsFile = path.join(__dirname, '../targets.json');
    const content = fs.readFileSync(targetsFile,  {encoding: 'utf8'});

    const parsed = JSON.parse(content);
    if (!parsed || !parsed.nodes) {
        throw new Error('Invalid targets file');
    }

    return parsed.nodes;
};

/**
 * Attempts to connect to each of the targets in the targets array.
 *
 * This function does not require that all connections succeed. The result of each connection attempt is returned. It is
 * up to the caller to decide how to handle the results. Either check that all attempts succeeded or filter out the
 * failed connections.
 *
 * @param targets The targets acquired by calling {@link loadTargets} or otherwise.
 * @param timeout The timeout in milliseconds for each connection attempt.
 * @returns An array of {(PromiseSettledResult<TryCpClient>)} for the connection attempts.
 */
export const connectTargets = async (targets: string[], timeout: number = 5_000): Promise<PromiseSettledResult<TryCpClient>[]> => {
    return await Promise.allSettled(targets.map(async (target) => {
        return await TryCpClient.create(new URL(target), 5_000);
    }));
};

/**
 * Filters out the successful connections from the attempted connections.
 *
 * @param attemptedConnections The result of calling {@link connectTargets} or otherwise.
 * @returns An array of {@link TryCpClient} instances for the TryCP servers we were able to connect to.
 */
export const filterConnectedTargets = (attemptedConnections: PromiseSettledResult<TryCpClient>[]): TryCpClient[] => {
    return attemptedConnections.map((result) => {
        if (result.status === 'fulfilled') {
            return result.value;
        } else {
            console.error(result.reason);
        }
    }).filter((client) => client);
};

/**
 * Convenience function that loads the targets, connects to them, and filters for the successful connections.
 *
 * @returns An array of {@link TryCpClient} instances for the TryCP servers we were able to connect to.
 */
export const tryConnectAllTargets = async (): Promise<TryCpClient[]> => {
    const targets = loadTargets();
    const attemptedConnections = await connectTargets(targets);
    return filterConnectedTargets(attemptedConnections);
};
