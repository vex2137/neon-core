import { NeonCoreGame } from './game';

const canvas = document.getElementById('game-canvas') as HTMLCanvasElement | null;
if (canvas) {
  const game = new NeonCoreGame(canvas);
  game.start();
}
