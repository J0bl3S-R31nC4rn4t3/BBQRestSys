import { createRouter, createWebHistory } from 'vue-router';
import { useAuth } from '../stores/authStore';

import Dashboard from '../views/Dashboard.vue';
import Cashier from '../views/Cashier.vue';
import Inventory from '../views/Inventory.vue';
import Staff from '../views/Staff.vue';
import Logs from '../views/Logs.vue';
import Schedule from '../views/Schedule.vue';
import PrepStation from '../views/PrepStation.vue';
import Login from '../authStore/Login.vue'; 
import CustomerMenu from '../ClientKiosk/CustomerMenu.vue';
import Settings from '../views/Settings.vue';

const routes = [
  { 
    path: '/login', 
    component: Login, 
    meta: { public: true } // NEW: Explicitly mark as public
  },
  { 
    path: '/menu', // NEW: The Public Kiosk Route
    component: CustomerMenu, 
    meta: { public: true } 
  },
  { 
    path: '/dashboard', 
    component: Dashboard, 
    meta: { roles: ['Admin'] } 
  },
  { 
    path: '/cashier', 
    component: Cashier, 
    meta: { roles: ['Staff'] } 
  },
  { 
    path: '/inventory', 
    component: Inventory, 
    meta: { roles: ['Admin'] } 
  },
  { 
    path: '/prep', 
    component: PrepStation, 
    meta: { roles: ['Staff'] } 
  },
  { 
    path: '/schedule', 
    component: Schedule, 
    meta: { roles: ['Admin'] } 
  },
  { 
    path: '/staff', 
    component: Staff, 
    meta: { roles: ['Admin'] } 
  },
  { 
    path: '/logs', 
    component: Logs, 
    meta: { roles: ['Admin'] } 
  },
  { 
    path: '/settings', 
    component: Settings, 
    meta: { roles: ['Admin'] } 
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.beforeEach((to, _from, next) => {
  const { isAuthenticated, hasAccess } = useAuth();

  // 1. If not logged in and trying to access a private page, redirect to login
  if (!isAuthenticated.value && !to.meta.public) {
    return next('/login');
  }

  // 2. If already logged in and trying to access the login page, redirect to dashboard
  if (isAuthenticated.value && to.path === '/login') {
    return next('/dashboard');
  }

  // 3. If the route is public (like /login), let them in
  if (to.meta.public) {
    return next();
  }

  // 4. Check if their role is in the allowed list for this specific route
  const requiredRoles = to.meta.roles as Array<'Admin' | 'Staff'>;
  
  if (requiredRoles && !hasAccess(requiredRoles)) {
    alert("Access Denied: You do not have permission to view this page.");
    return next(false); // Block navigation
  }

  // 5. If all checks pass, let them through
  next();
});

export default router;