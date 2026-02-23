import { runCli } from './src/cli';

runCli(process.argv, { defaultMode: 'x360' }).catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
