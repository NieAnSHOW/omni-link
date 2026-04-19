<template>
  <div class="app-layout">
    <aside class="sidebar">
      <div class="sidebar-header">
        <img src="/logo.png" alt="OmniLink" class="logo" />
        <h1>OmniLink</h1>
      </div>
      <nav>
        <router-link to="/links" class="nav-item" active-class="active">
          <span class="icon">🔗</span> 链接
        </router-link>
        <router-link to="/settings" class="nav-item" active-class="active">
          <span class="icon">⚙️</span> 设置
        </router-link>
      </nav>
    </aside>
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
}
.sidebar {
  width: 220px;
  background: #f8f9fa;
  border-right: 1px solid #e2e8f0;
  display: flex;
  flex-direction: column;
  padding: 16px 0;
}
.sidebar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 16px 16px;
  border-bottom: 1px solid #e2e8f0;
}
.logo { width: 28px; height: 28px; }
.sidebar-header h1 { font-size: 18px; margin: 0; }
nav { padding: 8px; }
.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  color: #475569;
  text-decoration: none;
  font-size: 14px;
}
.nav-item:hover { background: #e2e8f0; }
.nav-item.active { background: #6366f1; color: white; }
.main-content { flex: 1; overflow-y: auto; padding: 24px; }
</style>
