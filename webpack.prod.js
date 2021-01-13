const path = require('path');

const webpack = require('webpack');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const TerserPlugin = require('terser-webpack-plugin');

module.exports = {
  mode: 'production',
  devtool: 'source-map',

  entry: {
    bundle: './src/js/index.js',
    worker: './src/js/worker.js',
  },

  output: {
    filename: '[name].js',
    sourceMapFilename: '[name].map',
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

  optimization: {
    minimize: true,
    minimizer: [
      new TerserPlugin({
        exclude: /\.min\.js$/gi,
        parallel: true,
      }),
    ],
  },

  plugins: [
    new webpack.DefinePlugin({
      'process.env.NODE_ENV': JSON.stringify('production'),
      'process.env.ENCRUSTEDROOT': JSON.stringify('/encrusted/')
    }),

    new webpack.optimize.ModuleConcatenationPlugin(),

    new CopyWebpackPlugin({
      patterns: [
        { from: './src/dev.html', to: './index.html' },
        { from: './src/*.css', to: './[name].[ext]' },
        { from: './src/img/**.*', to: './img/[name].[ext]' },
      ],
    }),
  ]
};
