// Composables
import { RouteLocationNormalizedLoaded, createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    //component: () => import('@/layouts/default/Default.vue'),
    name: 'Home',
    component: () => import(/* webpackChunkName: "home" */ '@/views/Home.vue'),
    props: (route: RouteLocationNormalizedLoaded) => ({ searchText: route.query.searchText?.toString() ?? ""})
  },
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
})

export default router
