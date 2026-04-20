<template>
  <div class="link-card skeleton">
    <div class="link-header">
      <div class="skeleton-bar" style="width: 50px; height: 20px;"></div>
      <div class="skeleton-bar" style="width: 40px; height: 16px;"></div>
    </div>
    <div class="skeleton-bar" style="width: 70%; height: 18px; margin-bottom: 4px;"></div>
    <div class="skeleton-bar" style="width: 100%; height: 14px;"></div>
    <div class="link-footer">
      <div class="skeleton-bar" style="width: 80px; height: 14px;"></div>
    </div>
  </div>
</template>

<style scoped>
.skeleton {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 16px;
}
.link-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.link-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
}
.skeleton-bar {
  background: linear-gradient(90deg, #e2e8f0 25%, #f1f5f9 50%, #e2e8f0 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: 4px;
}
</style>
