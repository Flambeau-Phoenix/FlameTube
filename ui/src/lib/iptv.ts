import { invoke } from '@tauri-apps/api/core';
export interface LiveStream { url: string; label: string; referrer?: string; userAgent?: string }
export interface LiveChannel { id: string; tvgId: string; name: string; logo?: string; categories: string[]; country: string; languages: string[]; streams: LiveStream[] }
export interface LiveCatalogue { channels: LiveChannel[]; source: string; warning?: string }
export const getCatalogue = (refresh = false) => invoke<LiveCatalogue>('iptv_catalogue', { refresh });
export const playChannel = (id: string, streamIndex = 0) => invoke<void>('iptv_play', { id, streamIndex });
export const liveStatus = (id: string) => invoke<{width:number; height:number; streamIndex:number} | null>('iptv_status', { id });
