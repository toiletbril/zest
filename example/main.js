const endpoint = `http://localhost:6969/api/v1/music`;

let totalDuration = 0;

let mediaSource;
let sourceBuffer;

function fetchAndPlayMusic(trackName, chunkIndex = 0) {
    const audioPlayer = document.getElementById('audio-player');
    const url = endpoint + `/get?name=${encodeURIComponent(trackName)}&chunk=${chunkIndex}`;

    mediaSource = new MediaSource();
    audioPlayer.src = URL.createObjectURL(mediaSource);

    document.getElementById("track-name").textContent = trackName;

    mediaSource.addEventListener('sourceopen', () => {
        sourceBuffer = mediaSource.addSourceBuffer('audio/mpeg');
        fetchAndAppendChunk(sourceBuffer, mediaSource, url);
    });

    mediaSource.addEventListener('error', error => {
        console.error('Error opening media source:', error);
    });

    mediaSource.addEventListener('sourceended', () => {
        audioPlayer.duration = totalDuration;
    });
}

function resetPlayback() {
    totalDuration = 0;
    if (mediaSource) {
        if (sourceBuffer) {
            sourceBuffer.removeEventListener('updateend', handleUpdateEnd);
            mediaSource.removeSourceBuffer(sourceBuffer);
        }
        if (mediaSource.readyState === 'open') {
            mediaSource.endOfStream();
        }
    }
}

function fetchAndAppendChunk(sourceBuffer, mediaSource, url) {
    fetch(url)
        .then(response => {
            if (!response.ok) {
                throw new Error('Error retrieving music chunk');
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
        .catch(error => console.error('Error fetching music chunk:', error));
}

async function appendNextChunk(sourceBuffer, mediaSource, url, arrayBuffer) {
    sourceBuffer.appendBuffer(arrayBuffer);

    const isLastChunk = arrayBuffer.byteLength < 1024 * 512;
    if (!isLastChunk) {
        const nextChunkIndex = parseInt(new URL(url).searchParams.get('chunk')) + 1;
        const nextUrl = url.replace(/chunk=\d+/, `chunk=${nextChunkIndex}`);
        const duration = await getChunkDuration(arrayBuffer);
        totalDuration += duration;
        mediaSource.duration = totalDuration;
        fetchAndAppendChunk(sourceBuffer, mediaSource, nextUrl);
    }

    sourceBuffer.addEventListener('updateend', handleUpdateEnd);
}

function handleUpdateEnd() {
    const audioPlayer = document.getElementById('audio-player');
    audioPlayer.duration = mediaSource.duration;
}

function getChunkDuration(arrayBuffer) {
    const audioContext = new AudioContext();
    return audioContext.decodeAudioData(arrayBuffer)
        .then(decodedData => decodedData.duration)
        .catch(error => {
            console.error('Error decoding audio data:', error);
            return 0;
        });
}

function playTrack(trackName) {
    resetPlayback();
    fetchAndPlayMusic(trackName);
}

let allTracks = [];

function fetchTrackNames() {
    fetch(endpoint + '/all')
        .then(response => response.json())
        .then(trackNames => {
            allTracks = trackNames;
            updateMusicList(allTracks);
        })
        .catch(error => console.error('Error fetching track names:', error));
}

const searchInput = document.getElementById('search-input');

function searchTracks(event) {
    const searchTerm = searchInput.value.trim().toLowerCase();
    if (searchTerm === '') {
        updateMusicList(allTracks);
    } else if (event.key === 'Enter') {
        const filteredTracks = allTracks.filter(track => track.toLowerCase().includes(searchTerm));
        updateMusicList(filteredTracks);
    }
}

function updateMusicList(tracks) {
    const tracksDiv = document.getElementById('tracks');
    tracksDiv.innerHTML = '';

    tracks.forEach(trackName => {
        const trackLink = document.createElement('a');
        trackLink.href = '#';
        trackLink.className = "track";
        trackLink.innerText = trackName;
        trackLink.addEventListener('click', () => {
            event.preventDefault();
            playTrack(trackName);
        });

        const trackItem = document.createElement('div');
        trackItem.appendChild(trackLink);
        tracksDiv.appendChild(trackItem);
    });
}

searchInput.addEventListener('input', searchTracks);
searchInput.addEventListener('keyup', searchTracks);

fetchTrackNames();