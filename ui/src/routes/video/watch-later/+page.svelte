<script lang="ts">
	import VideoListPage from '$lib/components/VideoListPage.svelte';
	import { YT_WATCH_LATER } from '$lib/api';
	import { t } from '$lib/i18n.svelte';
</script>

<VideoListPage id={YT_WATCH_LATER} title={t('nav.watch_later')} />
