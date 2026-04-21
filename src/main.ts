import { mount } from 'svelte';
import App from './components/App.svelte';
import { sortBy } from './utils';

Array.prototype.sortBy = function (cfg) {
  return sortBy(this, cfg);
};

import { init, getLocaleFromNavigator, addMessages } from 'svelte-i18n';
import './app.css';

import en from './i18n/en.json';
import fr from './i18n/fr.json';

addMessages('en', en);
addMessages('fr', fr);

init({
  fallbackLocale: 'en',
  initialLocale: getLocaleFromNavigator(),
});

const app = mount(App, {
  target: document.body,
});

export default app;
