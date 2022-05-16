const { DeckGL, H3HexagonLayer, MapController, PathLayer } = deck;

class MyMapController extends MapController {
    handleEvent(event) {
        if (event.type === "panmove" || event.type === "wheel" || event.type === "pinchmove") {
            let v = deckgl.viewManager._viewports[0]
            let lat = v.latitude
            let lon = v.longitude
            let zoom = Math.floor(v.zoom)

            document.getElementById('coordinate-info').innerHTML = `${lat.toFixed(2)}, ${lon.toFixed(2)}`

            let center_h3 = h3.geoToH3(lat, lon, 4)
            let ring_sizes = [6, 4, 2, 1, 1, 0, 0, 0]
            let visible_h3s = h3.kRing(center_h3, ring_sizes[zoom - 7])

            for (let i = 0; i < vismap.length; i++) { 
                vismap[i] = false
            }
            var changed = false
            visible_h3s.forEach(visible_h3 => {
                if (chunk_map[visible_h3] != undefined) {
                    vismap[chunk_map[visible_h3]] = true
                    changed = true
                }
            })

            renderLayer()
        }
        super.handleEvent(event)
    }
}

const deckgl = new DeckGL({
    mapStyle: 'https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json',
    controller: { type: MyMapController },
    // controller: true,
    initialViewState: {
        longitude: 12.6,
        latitude: 55.6,
        zoom: 10,
        minZoom: 7,
        maxZoom: 15,
        pitch: 40.5,
    },
});

const days = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]

const colors = chroma.scale(['#59bfd9','#d9ae43']).mode('lch').colors(20).map(c => chroma(c).rgb().concat(160))

const OPTIONS = ['time'];
const options = {};

OPTIONS.forEach(key => {
    document.getElementById(key).oninput = renderLayer;
});

document.getElementById("now-button").addEventListener('click', e => setToCurrentTime())

function setToCurrentTime() {
    let now = new Date();
    let day = now.getDay()
    let hour = now.getHours()
    let value = (day * 24) + hour

    options["time"] = value;
    document.getElementById('time-value').innerHTML = value;
    document.getElementById('time').value = value
    document.getElementById('time-label').innerHTML = `${days[day]}, ${hour}:00 - ${hour + 1}:00`;
    renderLayer()
}

const dataChunks = []
var chunk_map = {}
var vismap = []

function onNewDataArrive(chunk) {
    dataChunks.push(chunk)
    console.log('new chunk arrived', dataChunks.length)
    renderLayer()
}

function renderLayer() {
    OPTIONS.forEach(key => {
        const value = +document.getElementById(key).value;
        document.getElementById(key + '-value').innerHTML = value;
        options[key] = parseInt(value);
        if (key === "time") {
            document.getElementById('time-label').innerHTML = `${days[Math.floor(value / 24)]}, ${(value % 24)}:00 - ${(value % 24) + 1}:00`;
        }
    });

    const layers = dataChunks.map((chunk, chunkIndex) => new H3HexagonLayer({
        id: `chunk-${chunkIndex}`,
        visible: vismap[chunkIndex],
        data: chunk,
        pickable: false,
        wireframe: false,
        filled: true,
        extruded: true,
        elevationScale: 40,
        getHexagon: d => d.h3,
        getFillColor: d => colors[d.color_index[options['time']]],
        getElevation: d => d.freq[options['time']],
        updateTriggers: {
            getElevation: vismap[chunkIndex] && [options['time']],
            getFillColor: vismap[chunkIndex] && [options['time']],
        }
    }))

    deckgl.setProps({ layers })
}

renderLayer();

// load h3 groups and incrementally load the data for each of them
d3.json("h3.json").then(data => {
    return data.flat().map(elem => {
        return {
            h3: elem
        }
    })
    // return [{
    //     h3: '841f059ffffffff'
    // }]
}).then(groups => {
    groups.forEach(group => {
        d3.json(`h3/${group.h3}.json`).then(data => {
            return data.map(d => {
                return {
                    h3: d.h3,
                    freq: Float32Array.from(d.freq),
                    color_index: Int8Array.from(d.freq.map(f => {
                        if (f > 0) {
                            return Math.min(Math.floor(f), 19.0)
                        } else { return 0 }
                    })),
                    type: d.type
                }
            })
        }).then(data => {
            chunk_map[group.h3] = dataChunks.length
            vismap[dataChunks.length] = false
            onNewDataArrive(data)
        })
    })
})

