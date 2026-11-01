import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';

export function load() {
	if (browser && localStorage.getItem('flametube_mode') !== 'music') redirect(307, '/video');
}
