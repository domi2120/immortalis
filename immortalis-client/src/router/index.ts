// Composables
import { RouteLocationNormalizedLoaded, createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    component: () => import('@/layouts/default/Default.vue'),
    children: [
      {
        path: '',
        name: 'Home',
        // route level code-splitting
        // this generates a separate chunk (about.[hash].js) for this route
        // which is lazy-loaded when the route is visited.
        component: () => import(/* webpackChunkName: "home" */ '@/views/Home.vue'),
      },
    ],
    name: 'Home',
  },
  {
    path: "/search",
    component: () => import(/* webpackChunkName: "home" */ '@/views/Search.vue'),
    props: (route: RouteLocationNormalizedLoaded) => ({ searchText: route.query.searchText?.toString() ?? ""})
  },
  {
    path: "/scheduling",
    component: () => import("@/views/Scheduling.vue"),
  },
  {
    path: "/tracked-collections",
    component: () => import("@/views/TrackedCollections.vue"),
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
})

export default router
