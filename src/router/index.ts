import { createRouter, createWebHashHistory } from 'vue-router';

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/notes' },
    { path: '/notes', name: 'notes', component: () => import('../views/NotesView.vue') },
    { path: '/notes/:id', name: 'note-detail', component: () => import('../views/NotesView.vue'), props: true },
    { path: '/settings', name: 'settings', component: () => import('../views/SettingsView.vue') },
  ],
});

export default router;
