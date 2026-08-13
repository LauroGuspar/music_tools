import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { PlayIcon, PauseIcon, CloseIcon, MusicIcon, VolumeIcon } from "./Icons";

export type AudioTrackInfo = {
  filePath: string;
  title: string;
  artists?: string[];
};

type AudioPlayerProps = {
  activeTrack: AudioTrackInfo | null;
  onClose: () => void;
};

export function AudioPlayer({ activeTrack, onClose }: AudioPlayerProps) {
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [volume, setVolume] = useState(0.8);
  const [playbackError, setPlaybackError] = useState<string | null>(null);

  // Web Audio API References
  const audioCtxRef = useRef<AudioContext | null>(null);
  const audioBufferRef = useRef<AudioBuffer | null>(null);
  const sourceNodeRef = useRef<AudioBufferSourceNode | null>(null);
  const gainNodeRef = useRef<GainNode | null>(null);

  const startTimeRef = useRef<number>(0);
  const pauseOffsetRef = useRef<number>(0);
  const animFrameRef = useRef<number | null>(null);
  const volumeRef = useRef<number>(volume);
  volumeRef.current = volume;

  // Obtenemos o inicializamos la instancia única de AudioContext
  const getAudioContext = (): AudioContext => {
    if (!audioCtxRef.current) {
      const AudioCtx =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
      audioCtxRef.current = new AudioCtx();
    }
    if (audioCtxRef.current.state === "suspended") {
      audioCtxRef.current.resume();
    }
    return audioCtxRef.current;
  };

  const stopSourceNode = () => {
    if (sourceNodeRef.current) {
      try {
        sourceNodeRef.current.onended = null;
        sourceNodeRef.current.stop();
        sourceNodeRef.current.disconnect();
      } catch {
        // Ignorar si ya estaba detenido
      }
      sourceNodeRef.current = null;
    }
  };

  const cancelProgressLoop = () => {
    if (animFrameRef.current !== null) {
      cancelAnimationFrame(animFrameRef.current);
      animFrameRef.current = null;
    }
  };

  const startProgressLoop = () => {
    cancelProgressLoop();

    const loop = () => {
      if (!audioCtxRef.current || !audioBufferRef.current) return;
      const elapsed = audioCtxRef.current.currentTime - startTimeRef.current;
      const dur = audioBufferRef.current.duration;

      if (elapsed >= dur) {
        setCurrentTime(dur);
        setIsPlaying(false);
        pauseOffsetRef.current = 0;
        cancelProgressLoop();
      } else {
        setCurrentTime(Math.max(0, elapsed));
        animFrameRef.current = requestAnimationFrame(loop);
      }
    };

    animFrameRef.current = requestAnimationFrame(loop);
  };

  const playBufferAt = (offset: number) => {
    if (!audioBufferRef.current) return;
    const ctx = getAudioContext();

    stopSourceNode();

    const gainNode = ctx.createGain();
    gainNode.gain.value = volumeRef.current;
    gainNode.connect(ctx.destination);
    gainNodeRef.current = gainNode;

    const source = ctx.createBufferSource();
    source.buffer = audioBufferRef.current;
    source.connect(gainNode);

    const safeOffset = Math.max(0, Math.min(offset, audioBufferRef.current.duration));
    startTimeRef.current = ctx.currentTime - safeOffset;
    pauseOffsetRef.current = safeOffset;

    source.onended = () => {
      if (
        audioCtxRef.current &&
        audioCtxRef.current.currentTime - startTimeRef.current >=
          (audioBufferRef.current?.duration || 0) - 0.1
      ) {
        setIsPlaying(false);
        pauseOffsetRef.current = 0;
        setCurrentTime(audioBufferRef.current?.duration || 0);
        cancelProgressLoop();
      }
    };

    source.start(0, safeOffset);
    sourceNodeRef.current = source;
    setIsPlaying(true);
    startProgressLoop();
  };

  const pauseAudio = () => {
    if (!audioCtxRef.current || !audioBufferRef.current) return;
    const elapsed = audioCtxRef.current.currentTime - startTimeRef.current;
    pauseOffsetRef.current = Math.max(0, Math.min(elapsed, audioBufferRef.current.duration));
    setCurrentTime(pauseOffsetRef.current);

    stopSourceNode();
    cancelProgressLoop();
    setIsPlaying(false);
  };

  useEffect(() => {
    setPlaybackError(null);
    let isCancelled = false;

    if (activeTrack) {
      (async () => {
        try {
          stopSourceNode();
          cancelProgressLoop();
          setIsPlaying(false);
          setCurrentTime(0);
          setDuration(0);
          pauseOffsetRef.current = 0;
          audioBufferRef.current = null;

          // Cargar binario vía IPC nativo ultra rápido
          const rawBuffer = await invoke<ArrayBuffer>("read_file_binary", {
            path: activeTrack.filePath,
          });
          if (isCancelled) return;

          const ctx = getAudioContext();
          // Decodificación Web Audio API: extrae única y exclusivamente el audio PCM real (ignora portadas ID3)
          const decodedBuffer = await ctx.decodeAudioData(rawBuffer.slice(0));
          if (isCancelled) return;

          audioBufferRef.current = decodedBuffer;
          setDuration(decodedBuffer.duration);
          setPlaybackError(null);

          // Iniciar reproducción desde 0:00 exactos
          playBufferAt(0);
        } catch (err) {
          if (!isCancelled) {
            console.error("Error al decodificar audio WebAudio API:", err);
            setIsPlaying(false);
            setPlaybackError("No se pudo decodificar este archivo de audio.");
          }
        }
      })();
    } else {
      stopSourceNode();
      cancelProgressLoop();
      setIsPlaying(false);
      setCurrentTime(0);
      setDuration(0);
      pauseOffsetRef.current = 0;
      audioBufferRef.current = null;
    }

    return () => {
      isCancelled = true;
      stopSourceNode();
      cancelProgressLoop();
    };
  }, [activeTrack]);

  const togglePlay = () => {
    if (!activeTrack || !audioBufferRef.current) return;
    if (isPlaying) {
      pauseAudio();
    } else {
      if (pauseOffsetRef.current >= audioBufferRef.current.duration - 0.1) {
        pauseOffsetRef.current = 0;
      }
      playBufferAt(pauseOffsetRef.current);
    }
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newTime = parseFloat(e.target.value);
    setCurrentTime(newTime);
    pauseOffsetRef.current = newTime;

    if (isPlaying && audioBufferRef.current) {
      playBufferAt(newTime);
    }
  };

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newVol = parseFloat(e.target.value);
    setVolume(newVol);
    volumeRef.current = newVol;
    if (gainNodeRef.current) {
      gainNodeRef.current.gain.value = newVol;
    }
  };

  const formatTime = (timeInSec: number) => {
    if (isNaN(timeInSec) || timeInSec <= 0) return "0:00";
    const minutes = Math.floor(timeInSec / 60);
    const seconds = Math.floor(timeInSec % 60);
    return `${minutes}:${seconds < 10 ? "0" : ""}${seconds}`;
  };

  if (!activeTrack) return null;

  return (
    <div className="audio-player-dock" role="region" aria-label="Reproductor de audio">
      <div className="audio-player-info">
        <span
          className="audio-player-title"
          title={activeTrack.title}
          style={{ display: "inline-flex", alignItems: "center", gap: "6px" }}
        >
          <MusicIcon style={{ color: "var(--accent)" }} />
          <span>{activeTrack.title}</span>
        </span>
        {playbackError ? (
          <span style={{ color: "#eb5757", fontSize: "0.76rem" }}>{playbackError}</span>
        ) : activeTrack.artists && activeTrack.artists.length > 0 ? (
          <span className="audio-player-artist" title={activeTrack.artists.join(", ")}>
            {activeTrack.artists.join(", ")}
          </span>
        ) : null}
      </div>

      <div className="audio-player-controls">
        <button
          type="button"
          className="audio-player-btn play-pause-btn"
          onClick={togglePlay}
          title={isPlaying ? "Pausar" : "Reproducir"}
          disabled={Boolean(playbackError) || !audioBufferRef.current}
        >
          {isPlaying ? <PauseIcon /> : <PlayIcon />}
        </button>

        <span className="audio-player-time">{formatTime(currentTime)}</span>

        <input
          type="range"
          className="audio-player-seekbar"
          min={0}
          max={duration || 100}
          step={0.1}
          value={currentTime}
          onChange={handleSeek}
          disabled={Boolean(playbackError) || !audioBufferRef.current}
        />

        <span className="audio-player-time">{formatTime(duration)}</span>
      </div>

      <div className="audio-player-volume">
        <VolumeIcon
          className="volume-icon"
          style={{ fontSize: "1.1rem", color: "rgba(244, 247, 245, 0.7)" }}
        />
        <input
          type="range"
          className="audio-player-volbar"
          min={0}
          max={1}
          step={0.05}
          value={volume}
          onChange={handleVolumeChange}
          title={`Volumen: ${Math.round(volume * 100)}%`}
        />
      </div>

      <button
        type="button"
        className="audio-player-btn close-player-btn"
        onClick={() => {
          stopSourceNode();
          cancelProgressLoop();
          onClose();
        }}
        title="Cerrar reproductor"
      >
        <CloseIcon />
      </button>
    </div>
  );
}
