import { useState, useEffect, useRef } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { PlayIcon, PauseIcon, CloseIcon } from "./Icons";

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

  useEffect(() => {
    if (activeTrack && audioRef.current) {
      const mediaSrc = convertFileSrc(activeTrack.filePath);
      audioRef.current.src = mediaSrc;
      audioRef.current.volume = volume;
      audioRef.current
        .play()
        .then(() => setIsPlaying(true))
        .catch((err) => {
          console.error("Audio playback error:", err);
          setIsPlaying(false);
        });
    } else {
      if (audioRef.current) {
        audioRef.current.pause();
        audioRef.current.src = "";
      }
      setIsPlaying(false);
      setCurrentTime(0);
      setDuration(0);
    }
  }, [activeTrack]);

  const togglePlay = () => {
    if (!audioRef.current || !activeTrack) return;
    if (isPlaying) {
      audioRef.current.pause();
      setIsPlaying(false);
    } else {
      audioRef.current
        .play()
        .then(() => setIsPlaying(true))
        .catch((err) => console.error("Play error:", err));
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
      />

      <div className="audio-player-info">
        <span className="audio-player-title" title={activeTrack.title}>
          🎵 {activeTrack.title}
        </span>
        {activeTrack.artists && activeTrack.artists.length > 0 ? (
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
        />

        <span className="audio-player-time">{formatTime(duration)}</span>
      </div>

      <div className="audio-player-volume">
        <span className="volume-icon">🔊</span>
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
