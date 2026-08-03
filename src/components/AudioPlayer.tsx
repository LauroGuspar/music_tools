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
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [volume, setVolume] = useState(0.8);
  const [playbackError, setPlaybackError] = useState<string | null>(null);
  const currentBlobUrlRef = useRef<string | null>(null);

  useEffect(() => {
    setPlaybackError(null);

    // Revocar blob URL previo si existía
    if (currentBlobUrlRef.current) {
      URL.revokeObjectURL(currentBlobUrlRef.current);
      currentBlobUrlRef.current = null;
    }

    let isSubscribed = true;

    if (activeTrack && audioRef.current) {
      const loadAudioFile = async () => {
        try {
          // Leer bytes del archivo local via IPC Tauri para omitir restricciones de esquemas de asset://
          const fileBytes = await invoke<number[]>("read_file_binary", {
            path: activeTrack.filePath,
          });

          if (!isSubscribed) return;

          const ext = activeTrack.filePath.toLowerCase();
          const mimeType = ext.endsWith(".flac")
            ? "audio/flac"
            : ext.endsWith(".wav")
            ? "audio/wav"
            : ext.endsWith(".m4a")
            ? "audio/mp4"
            : "audio/mpeg";

          const blob = new Blob([new Uint8Array(fileBytes)], { type: mimeType });
          const blobUrl = URL.createObjectURL(blob);
          currentBlobUrlRef.current = blobUrl;

          if (audioRef.current && isSubscribed) {
            audioRef.current.src = blobUrl;
            audioRef.current.volume = volume;
            audioRef.current.load();

            audioRef.current
              .play()
              .then(() => {
                if (isSubscribed) {
                  setIsPlaying(true);
                  setPlaybackError(null);
                }
              })
              .catch((err) => {
                console.warn("Audio playback exception:", err);
                if (isSubscribed) {
                  setIsPlaying(false);
                  setPlaybackError("El reproductor del navegador no pudo reproducir este formato.");
                }
              });
          }
        } catch (err) {
          console.error("Error al cargar binario del archivo de audio:", err);
          if (isSubscribed) {
            setIsPlaying(false);
            setPlaybackError("No se pudo leer el archivo desde el disco local.");
          }
        }
      };

      loadAudioFile();
    } else {
      if (audioRef.current) {
        audioRef.current.pause();
        audioRef.current.removeAttribute("src");
        audioRef.current.load();
      }
      setIsPlaying(false);
      setCurrentTime(0);
      setDuration(0);
    }

    return () => {
      isSubscribed = false;
      if (currentBlobUrlRef.current) {
        URL.revokeObjectURL(currentBlobUrlRef.current);
        currentBlobUrlRef.current = null;
      }
    };
  }, [activeTrack]);

  const togglePlay = () => {
    if (!audioRef.current || !activeTrack) return;
    if (isPlaying) {
      audioRef.current.pause();
      setIsPlaying(false);
    } else {
      audioRef.current
        .play()
        .then(() => {
          setIsPlaying(true);
          setPlaybackError(null);
        })
        .catch((err) => {
          console.warn("Play error:", err);
          setPlaybackError("No se pudo iniciar la reproducción del archivo.");
        });
    }
  };

  const handleTimeUpdate = () => {
    if (audioRef.current) {
      setCurrentTime(audioRef.current.currentTime);
      setDuration(audioRef.current.duration || 0);
    }
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newTime = parseFloat(e.target.value);
    if (audioRef.current) {
      audioRef.current.currentTime = newTime;
      setCurrentTime(newTime);
    }
  };

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newVol = parseFloat(e.target.value);
    setVolume(newVol);
    if (audioRef.current) {
      audioRef.current.volume = newVol;
    }
  };

  const formatTime = (timeInSec: number) => {
    if (isNaN(timeInSec) || timeInSec === 0) return "0:00";
    const minutes = Math.floor(timeInSec / 60);
    const seconds = Math.floor(timeInSec % 60);
    return `${minutes}:${seconds < 10 ? "0" : ""}${seconds}`;
  };

  if (!activeTrack) return null;

  return (
    <div className="audio-player-dock" role="region" aria-label="Reproductor de audio">
      <audio
        ref={audioRef}
        onTimeUpdate={handleTimeUpdate}
        onEnded={() => setIsPlaying(false)}
        onLoadedMetadata={handleTimeUpdate}
        onError={() => {
          setIsPlaying(false);
          setPlaybackError("El reproductor no puede decodificar este archivo local.");
        }}
        preload="auto"
      />

      <div className="audio-player-info">
        <span className="audio-player-title" title={activeTrack.title} style={{ display: "inline-flex", alignItems: "center", gap: "6px" }}>
          <MusicIcon style={{ color: "#79e5a7" }} />
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
          disabled={Boolean(playbackError)}
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
          disabled={Boolean(playbackError)}
        />

        <span className="audio-player-time">{formatTime(duration)}</span>
      </div>

      <div className="audio-player-volume">
        <VolumeIcon className="volume-icon" style={{ fontSize: "1.1rem", color: "rgba(244, 247, 245, 0.7)" }} />
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
        onClick={onClose}
        title="Cerrar reproductor"
      >
        <CloseIcon />
      </button>
    </div>
  );
}
