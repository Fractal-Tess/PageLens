<script lang="ts">
  import Router from 'svelte-spa-router';
  
  import Dashboard from './routes/Dashboard.svelte';
  import AnalysisRedirect from './routes/AnalysisRedirect.svelte';
  import Simple from './routes/Simple.svelte';
  import Advanced from './routes/Advanced.svelte';
  import Profiles from './routes/Profiles.svelte';
  import History from './routes/History.svelte';
  import Report from './routes/Report.svelte';
  import Run from './routes/Run.svelte';
  import RunPage from './routes/RunPage.svelte';

  const routes = {
    '/': Dashboard,
    '/#analysis': AnalysisRedirect,
    '/#simple': Simple,
    '/#advanced': Advanced,
    '/#profiles': Profiles,
    '/#history': History,
    '/report/:id': Report,
    '/run/:id': Run,
    '/run/:id/page/:pageId': RunPage,
  };
</script>

<Router {routes} />
