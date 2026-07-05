// Welcome-greeting text + speech — port of waitress/shared/voice.py. The
// Python app needed a whole pluggable-backend system (pyttsx3 on desktop,
// silent text-only everywhere else) because Flet's Python runtime has no
// built-in TTS. The browser engine underneath Tauri's webview always has one
// (Web Speech API), so this collapses to a much smaller wrapper — and it
// works identically on every OS, unlike pyttsx3 which was desktop-only.

export function buildGreeting(restaurantName, specialTitles, starterCategoryNames) {
  const parts = [`Hello and welcome to ${restaurantName}!`];

  const specials = (specialTitles || []).filter(Boolean);
  if (specials.length === 1) {
    parts.push(`Today's special is ${specials[0]}.`);
  } else if (specials.length > 1) {
    const listed = specials.slice(0, -1).join(", ") + ` and ${specials[specials.length - 1]}`;
    parts.push(`Today's specials are ${listed}.`);
  }

  const starters = (starterCategoryNames || []).filter(Boolean);
  if (starters.length) {
    let listed;
    if (starters.length <= 2) {
      listed = starters.join(" or ");
    } else {
      listed = starters.slice(0, -1).join(", ") + `, or ${starters[starters.length - 1]}`;
    }
    parts.push(`Would you like to start with some ${listed}?`);
  }

  parts.push("Take your time — just tap the menu whenever you're ready.");
  return parts.join(" ");
}

class WaitressVoice {
  constructor() {
    this.speaking = false;
    this.available = typeof window !== "undefined" && "speechSynthesis" in window;
  }

  speak(text, { muted = false, onDone } = {}) {
    if (!this.available || muted || !text) {
      onDone?.();
      return;
    }
    try {
      window.speechSynthesis.cancel(); // never overlap two greetings
      const utterance = new SpeechSynthesisUtterance(text);
      utterance.rate = 0.95;
      const voices = window.speechSynthesis.getVoices();
      const female = voices.find((v) => /female|samantha|zira|victoria/i.test(v.name));
      if (female) utterance.voice = female;
      utterance.onend = () => onDone?.();
      utterance.onerror = () => onDone?.();
      this.speaking = true;
      window.speechSynthesis.speak(utterance);
    } catch {
      onDone?.();
    }
  }

  stop() {
    if (this.available) {
      try {
        window.speechSynthesis.cancel();
      } catch {
        /* ignore */
      }
    }
    this.speaking = false;
  }
}

export const voice = new WaitressVoice();
