import { createRouter, createWebHashHistory } from 'vue-router';

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/links' },
    { path: '/links', name: 'links', component: () => import('../views/LinksView.vue') },
    { path: '/tags', name: 'tags', component: () => import('../views/TagsView.vue') },
    { path: '/links/:id', name: 'content', component: () => import('../views/ContentView.vue'), props: true },
    { path: '/notes', name: 'notes', component: () => import('../views/NotesView.vue') },
    { path: '/settings', name: 'settings', component: () => import('../views/SettingsView.vue') },
  ],
});

export default router;
