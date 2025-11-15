const nextConfig = {
  // Убираем output: 'export' для dev режима
  // output: 'export', // Используется только для статического экспорта
  images: {
    unoptimized: true,
  },
  // Убираем assetPrefix для dev режима
  // assetPrefix: './', // Вызывает проблемы с путями в dev
  trailingSlash: true,
  webpack: (config: { resolve: { fallback: any; }; }) => {
    config.resolve.fallback = {
      ...(config.resolve.fallback || {}),
      fs: false,
      path: false,
    };
    return config;
  },
};

module.exports = nextConfig;
