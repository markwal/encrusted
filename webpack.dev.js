const path = require('path');

const webpack = require('webpack');
const CopyWebpackPlugin = require('copy-webpack-plugin');

module.exports = {
  mode: 'development',
  devtool: 'source-map',

  entry: {
    bundle: './src/js/index.js',
    worker: './src/js/worker.js'
  },

  output: {
    filename: '[name].js',
    path: path.join(__dirname, '/build'),
  },

  module: {
    rules: [
      {
        test: /.jsx?$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
          options: {
            presets: ['@babel/preset-react'],
          }
        }
      }
    ]
  },

  plugins: [
    new webpack.DefinePlugin({
      'process.env.ENCRUSTEDROOT': JSON.stringify('/')
    }),

    new CopyWebpackPlugin({
      patterns: [
        { from: './src/dev.html', to: './index.html' },
        { from: './src/*.css', to: './[name].[ext]' },
        { from: './src/img/**.*', to: './img/[name].[ext]' },
      ],
    }),
  ],

  devServer: {
    historyApiFallback: {
      rewrites: [
        { from: /^\/run\/.+/, to: '/index.html' },
        { from: /./, to: '/404.html' }
      ]
    }
  }
};
