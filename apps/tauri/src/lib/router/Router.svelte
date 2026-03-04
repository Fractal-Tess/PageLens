<script lang="ts">
  import Router from 'svelte-spa-router'

  import Dashboard from '$lib/router/routes/Dashboard.svelte'
  import AnalysisRedirect from '$lib/router/routes/AnalysisRedirect.svelte'
  import Simple from '$lib/router/routes/Simple.svelte'
  import Advanced from '$lib/router/routes/Advanced.svelte'
  import Profiles from '$lib/router/routes/Profiles.svelte'
  import History from '$lib/router/routes/History.svelte'
  import Report from '$lib/router/routes/Report.svelte'
  import Run from '$lib/router/routes/Run.svelte'
  import RunPage from '$lib/router/routes/RunPage.svelte'

  const routes = {
    '/': Dashboard,
    '/#analysis': AnalysisRedirect,
    '/#simple': Simple,
    '/#advanced': Advanced,
    '/#profiles': Profiles,
    '/#history': History,
    '/report/:id': Report,
    '/run/:id': Run,
    '/run/:id/page/:pageId': RunPage
  }
</script>

<Router {routes} />
