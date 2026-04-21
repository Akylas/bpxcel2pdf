import cssnano from 'cssnano';

const production = process.env.NODE_ENV === 'production';

export default {
  plugins: [production && cssnano({ preset: 'default' })].filter(Boolean),
};
