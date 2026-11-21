// Procedural Web Audio synth mirroring synth_wav in src/main.rs

export type SoundKey = 'shoot' | 'hit' | 'boom' | 'dash' | 'lvlup';

export class SoundBank {
  private ctx: AudioContext | null = null;
  private buffers: Partial<Record<SoundKey, AudioBuffer>> = {};
  public shootCd = 0;

  public unlock(): void {
    if (!this.ctx) {
      const AudioCtx =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
      if (!AudioCtx) return;
      this.ctx = new AudioCtx();
      this.buffers.shoot = this.synthBuffer(460, 110, 0.07, 0.22, true);
      this.buffers.hit = this.synthBuffer(190, 55, 0.05, 0.25, false);
      this.buffers.boom = this.synthBuffer(135, 24, 0.24, 0.42, true);
      this.buffers.dash = this.synthBuffer(170, 540, 0.14, 0.3, false);
      this.buffers.lvlup = this.synthBuffer(330, 880, 0.28, 0.32, false);
    }
    if (this.ctx && this.ctx.state === 'suspended') {
      void this.ctx.resume();
    }
  }

  private synthBuffer(
    freqStart: number,
    freqEnd: number,
    duration: number,
    volume: number,
    square: boolean
  ): AudioBuffer {
    const ctx = this.ctx!;
    const sampleRate = 22050;
    const numSamples = Math.max(1, Math.floor(sampleRate * duration));
    const buffer = ctx.createBuffer(1, numSamples, sampleRate);
    const data = buffer.getChannelData(0);

    let phase = 0;
    for (let i = 0; i < numSamples; i++) {
      const t = i / numSamples;
      const freq = freqStart * Math.pow(freqEnd / freqStart, t);
      phase += freq / sampleRate;
      const wave = square
        ? phase - Math.floor(phase) < 0.5
          ? 1
          : -1
        : Math.sin(phase * Math.PI * 2);
      const env = Math.pow(1 - t, 1.6) * volume;
      data[i] = Math.max(-1, Math.min(1, wave * env));
    }
    return buffer;
  }

  public play(key: SoundKey, enabled: boolean, vol: number): void {
    if (!enabled) return;
    this.unlock();
    if (!this.ctx) return;
    const buf = this.buffers[key];
    if (!buf) return;

    const src = this.ctx.createBufferSource();
    const gain = this.ctx.createGain();
    src.buffer = buf;
    gain.gain.value = vol;
    src.connect(gain);
    gain.connect(this.ctx.destination);
    src.start();
  }
}
