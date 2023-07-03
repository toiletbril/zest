const endpoint = `http://localhost:6969/api/v1/music`;

let updatingBuffers = 0;
let totalDuration = 0;

const audioPlayer = document.getElementById("audio-player");

let mediaSource;
let sourceBuffer;

function fetchAndPlayMusic(trackName, chunkIndex = 0) {
    const url = endpoint + `/get?name=${encodeURIComponent(trackName)}&chunk=${chunkIndex}`;

    mediaSource = new MediaSource();
    audioPlayer.src = URL.createObjectURL(mediaSource);

    document.getElementById("track-name").textContent = trackName;

    mediaSource.addEventListener("sourceopen", () => {
        sourceBuffer = mediaSource.addSourceBuffer("audio/mpeg");
        fetchAndAppendChunk(sourceBuffer, mediaSource, url);
    });

    mediaSource.addEventListener("error", error => {
        console.error("Error opening media source:", error);
    });

    mediaSource.addEventListener("sourceended", () => {
        audioPlayer.duration = totalDuration;
    });
}

function resetPlayback() {
    totalDuration = 0;
    if (mediaSource) {
        if (sourceBuffer) {
            sourceBuffer.removeEventListener("updateend", handleUpdateEnd);
            mediaSource.removeSourceBuffer(sourceBuffer);
        }
        if (mediaSource.readyState === "open") {
            mediaSource.endOfStream();
        }
    }
}

function fetchAndAppendChunk(sourceBuffer, mediaSource, url) {
    fetch(url)
        .then(response => {
            if (!response.ok) {
                throw new Error("Error retrieving music chunk");
            }
            return response.arrayBuffer();
        })
        .then(arrayBuffer => {
            if (sourceBuffer.updating) {
                const appendInterval = setInterval(() => {
                    if (!sourceBuffer.updating) {
                        clearInterval(appendInterval);
                        appendNextChunk(sourceBuffer, mediaSource, url, arrayBuffer);
                    }
                }, 100);
            } else {
                appendNextChunk(sourceBuffer, mediaSource, url, arrayBuffer);
            }
        })
        .catch(error => console.error("Error fetching music chunk:", error));
}

const CHUNK_SIZE = 1024 * 128; // 128 kb

function appendNextChunk(sourceBuffer, mediaSource, url, arrayBuffer) {
    sourceBuffer.appendBuffer(arrayBuffer);
    const isLastChunk = arrayBuffer.byteLength < CHUNK_SIZE;

    if (!isLastChunk) {
        const nextChunkIndex = parseInt(new URL(url).searchParams.get("chunk")) + 1;
        const nextUrl = url.replace(/chunk=\d+/, `chunk=${nextChunkIndex}`);
        fetchAndAppendChunk(sourceBuffer, mediaSource, nextUrl);
    }

    updatingBuffers++;
    sourceBuffer.addEventListener("updateend", handleUpdateEnd);
}

function handleUpdateEnd() {
    updatingBuffers--;
    if (updatingBuffers === 0) {
        mediaSource.duration = totalDuration;
        audioPlayer.duration = mediaSource.duration;
    }
}

function getChunkDuration(arrayBuffer) {
    const audioContext = new AudioContext();
    return audioContext.decodeAudioData(arrayBuffer)
        .then(decodedData => decodedData.duration)
        .catch(error => {
            console.error("Error decoding audio data:", error);
            return 0;
        });
}

function playTrack(trackName) {
    resetPlayback();
    fetchAndPlayMusic(trackName);
}

let allTracks = [];

function fetchTrackNames() {
    fetch(endpoint + "/all")
        .then(response => response.json())
        .then(trackNames => {
            allTracks = trackNames;
            updateMusicList(allTracks);
        })
        .catch(error => console.error("Error fetching track names:", error));
}

const searchInput = document.getElementById("search-input");

function searchTracks(event) {
    const searchTerm = searchInput.value.trim().toLowerCase();
    if (searchTerm === "") {
        updateMusicList(allTracks);
    } else if (event.key === "Enter") {
        const filteredTracks = allTracks.filter(track => track.toLowerCase().includes(searchTerm));
        updateMusicList(filteredTracks);
    }
}

function updateMusicList(tracks) {
    const tracksDiv = document.getElementById("tracks");
    tracksDiv.innerHTML = "";

    tracks.forEach(trackName => {
        const trackLink = document.createElement("a");
        trackLink.href = "#";
        trackLink.className = "track";
        trackLink.innerText = trackName;
        trackLink.addEventListener("click", () => {
            event.preventDefault();
            playTrack(trackName);
        });

        const trackItem = document.createElement("div");
        trackItem.appendChild(trackLink);
        tracksDiv.appendChild(trackItem);
    });
}

window.onload = () => {
    fetchTrackNames();

    searchInput.addEventListener("input", searchTracks);
    searchInput.addEventListener("keyup", searchTracks);

    if (!MediaSource.isTypeSupported("audio/mpeg")) {
        alert("Your browser does not support decoding of audio/mpeg, and playing music will not work.\n\n" +
            "To fix this, please install a compatible decoder, such as ffmpeg.");
    }
};
