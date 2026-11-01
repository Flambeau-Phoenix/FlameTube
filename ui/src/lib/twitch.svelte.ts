import type { TwitchTarget } from './twitch';

// A route change unloads the remote player; keep the destination so Now Playing can reopen it.
export const twitchSession = $state({ target: null as TwitchTarget | null, returnPending: false });
