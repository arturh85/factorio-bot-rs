module.exports = {
  transpileDependencies: ["vuetify"],
  devServer: {
    proxy: {
      '^/api': {
        target: 'http://127.0.0.1:7123',
        ws: true,
        changeOrigin: true
      },
    }
  }
};
