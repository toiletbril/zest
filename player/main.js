const ZEST_PORT = 6969;
const ENDPOINT =
    "http://" +
    `${window.location.hostname}:${ZEST_PORT}` +
    "/api/v1/music";
const CHUNK_SIZE = 1024 * 128;

class AudioPlayer {
    constructor (audioPlayer, chunkSize) {
        this.audioPlayer = audioPlayer;
        this.chunkSize = chunkSize;
        this.shouldFetch = false;
        this.fetching = false;

        this.initialize();
    }

    initialize() {
        this.mediaSource = new MediaSource();
        this.totalDuration = 0;
        this.audioBuffer = null;
        this.audioQueue = [];

        this.setup();
        this.setupAudioPreprocessor();
    }

    setup() {
        this.audioPlayer.preload = "none";
        this.audioPlayer.autoplay = true;
        this.audioPlayer.src = URL.createObjectURL(this.mediaSource);
    }

    setupAudioPreprocessor() {
        this.mediaSource.addEventListener("sourceopen", () => {
            this.audioBuffer = this.mediaSource.addSourceBuffer("audio/mpeg");
            this.audioBuffer.mode = "sequence";
            this.audioBuffer.addEventListener("update", () => {
                if (this.audioQueue.length > 0 && this.audioBuffer && !this.audioBuffer.updating) {
                    this.audioBuffer.appendBuffer(this.audioQueue.shift());
                }
            });

            this.audioBuffer.addEventListener("error", (err) => {
                console.error("AudioBuffer error:", err);
            });
        });
    }

    resume() {
        if (this.audioPlayer.paused) {
            this.audioPlayer.play()
                .catch((err) => {
                    console.error("Error playing AudioBuffer:", err);
                });
        }
    }

    pause() {
        if (!this.audioPlayer.paused) {
            this.audioPlayer.pause()
                .catch((err) => {
                    console.error("Error pausing AudioBuffer:", err);
                });
        }
    }

    setPlayingTrackName(trackName) {
        document.title = trackName + " - Zest Player";
        document.getElementById("currentTrackName").textContent = trackName;
    }

    appendBuffer(buffer) {
        if (this.audioBuffer.updating || this.audioQueue.length > 0) {
            this.audioQueue.push(buffer);
        } else {
            this.audioBuffer.appendBuffer(buffer);
        }
    }

    fetchTrack(trackName) {
        const url = ENDPOINT + `/get?name=${encodeURIComponent(trackName)}&chunk=0`;

        let fetchChunks = (url) => new Promise((resolve, reject) => {
            if (!this.shouldFetch) {
                return resolve();;
            }

            fetch(url)
            .then((response) => {
                return response.arrayBuffer();
            })
            .then((arrayBuffer) => {
                const arrayBufferClone = structuredClone(arrayBuffer); // since extend duration consumes arrayBuffer
                this.extendDuration(arrayBufferClone);

                this.appendBuffer(arrayBuffer);

                const isLastChunk = arrayBuffer.byteLength < this.chunkSize;

                if (!isLastChunk) {
                    const nextChunkIndex = parseInt(new URL(url).searchParams.get("chunk")) + 1;
                    const nextUrl = url.replace(/chunk=\d+/, `chunk=${nextChunkIndex}`);

                    fetchChunks(nextUrl);
                    resolve();
                } else {
                    resolve();
                }
                })
                .catch((err) => {
                    console.error("Error appending music chunk:", err)
                    resolve();
                });
            });

            if (!this.fetching) {
                this.shouldFetch = true;
                this.fetching = true;

                fetchChunks(url).then(() => {
                    this.fetching = false;
                });
            }
        }

        extendDuration(arrayBuffer) {
        const audioContext = new AudioContext();

        return audioContext.decodeAudioData(arrayBuffer)
            .then((decodedData) => {
                this.totalDuration += decodedData.duration;
            })
            .catch(() => "hii :3");
    }

    async playTrack(trackName) {
        await this.reset();

        this.fetchTrack(trackName);
        this.setPlayingTrackName(trackName);
        this.resume();
        this.waitForDuration();
    }

    reset() {
        this.shouldFetch = false;

        const resetPlayer = () => {
            this.audioPlayer.src = "";
            this.audioPlayer.currentTime = 0;
            this.initialize();
        }

        return new Promise((resolve) => {
            const interval = setInterval(() => {
                if (this.fetching === false) {
                    clearInterval(interval);
                    resetPlayer();
                    resolve();
                }
            }, 200);
        });
    }

    waitForDuration() {
        const interval = setInterval(() => {
            if (!this.audioBuffer.updating) {
                clearInterval(interval);
                try {
                    this.mediaSource.duration = this.totalDuration;
                    this.audioPlayer.duration = this.totalDuration;
                } catch {}
            }
        }, 200)
    }
}

class MusicList {
    constructor (trackListDivElement, searchInputElement, player) {
        this.searchInputElement = searchInputElement;
        this.trackListDivElement = trackListDivElement;
        this.player = player;
        this.trackList = [];
    }

    fetchTrackList() {
        fetch(ENDPOINT + "/all")
            .then((response) => response.json())
            .then((trackNames) => {
                this.trackList = trackNames;
                this.updateTrackListElement(this.trackList);
            })
            .catch(error => console.error("Error fetching track names:", error));
    }

    updateTrackListElement(trackArray) {
        if (trackArray.length > 100) {
            trackArray = trackArray.slice(0, 100);
        }

        this.trackListDivElement.innerHTML = "";

        trackArray.forEach(trackName => {
            const trackLink = document.createElement("a");

            trackLink.href = "#";
            trackLink.className = "header-trackList-trackEntry";

            trackLink.innerText = trackName;

            trackLink.addEventListener("click", () => {
                event.preventDefault();
                this.player.playTrack(trackName);
            });

            const trackItem = document.createElement("div");

            trackItem.appendChild(trackLink);
            this.trackListDivElement.appendChild(trackItem);
        });
    }

    searchTracks(event) {
        const searchTerm = searchInput.value.trim().toLowerCase();
        if (searchTerm === "") {
            updateMusicList(allTracks);
        } else if (event.key === "Enter") {
            const filteredTracks = this.trackList
                .filter(track => track.toLowerCase().includes(searchTerm));

            this.updateTrackListElement(filteredTracks);
        }
    }
}

window.onload = () => {
    const audioPlayer = document.getElementById("audioControls");
    const searchInput = document.getElementById("searchInput");
    const trackList = document.getElementById("trackList");

    const zestPlayer = new AudioPlayer(audioPlayer, CHUNK_SIZE);
    const musicList = new MusicList(trackList, searchInput, zestPlayer);

    musicList.fetchTrackList();

    searchInput.addEventListener("input", musicList.searchTracks);
    searchInput.addEventListener("keyup", musicList.searchTracks);

    if (!MediaSource.isTypeSupported("audio/mpeg")) {
        alert(
            "Your browser does not support decoding of audio/mpeg, and playing music will not work.\n\n" +
            "To use this application, please install a compatible decoder, such as ffmpeg, or use Edge or Chromium.");
    }
}
