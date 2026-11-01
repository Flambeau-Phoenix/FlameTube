<script lang="ts">
	import VideoListPage from '$lib/components/VideoListPage.svelte';
	import { YT_PLAYLISTS } from '$lib/api';
	import { t } from '$lib/i18n.svelte';
</script>

<VideoListPage id={YT_PLAYLISTS} title={t('nav.your_playlists')} />
