<script lang="ts">
	import VideoListPage from '$lib/components/VideoListPage.svelte';
	import { YT_HISTORY } from '$lib/api';
	import { t } from '$lib/i18n.svelte';
</script>

<VideoListPage id={YT_HISTORY} title={t('nav.history')} />
