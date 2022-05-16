const {DeckGL, H3HexagonLayer, MapController, PathLayer} = deck;

class MyMapController extends MapController {
    handleEvent(event) {
        if (event.type === "pan" || event.type === "zoom") {
            let v = deckgl.viewManager._viewports[0]
            let lat = v.latitude
            let lon = v.longitude

            document.getElementById('coordinate-info').innerHTML = `${lat.toFixed(2)}, ${lon.toFixed(2)}`
        }
        super.handleEvent(event)
    }
}

const deckgl = new DeckGL({
    mapStyle: 'https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json',
    controller: {type: MyMapController},
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

const OPTIONS = ['time'];
const options = {};

OPTIONS.forEach(key => {
    document.getElementById(key).oninput = renderLayer;
});

document.getElementById("now-button").onclick = setToCurrentTime;
function setToCurrentTime() {
    let now = new Date();
    let day = now.getDay()
    let hour = now.getHours()
    let value = (day*24) + hour

    options["time"] = value;
    document.getElementById( 'time-value').innerHTML = value;
    document.getElementById('time').value = value
    document.getElementById('time-label').innerHTML = `${days[day]}, ${hour}:00 - ${hour + 1}:00`;
    renderLayer()
}

function getColor(value) {
    switch (value) {
        case 0:
            return [89, 191, 217, 120]
        case 1:
            return [217, 131, 111, 140]
        case 2: 
            return [78, 217, 168, 160]
        case 3:
            return [217, 174, 67, 180]
        default:
            return [100, 132, 217, 100]
    }
}

const dataChunks = []

function onNewDataArrive(chunk) {
    dataChunks.push(chunk)
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
        data: chunk,
        pickable: true,
        wireframe: false,
        filled: true,
        extruded: true,
        elevationScale: 20,
        getHexagon: d => d.h3,
        getFillColor: d => getColor(d.type),
        getElevation: d => d.freq[options['time']],
        updateTriggers: {
            getElevation: [options['time']]
        },
        getTooltip: d => `score = ${d.freq[options['time']]}`
    }))

    deckgl.setProps({layers})
}

renderLayer();

// load h3 groups and incrementally load the data for each of them
d3.json("h3.json").then(data => {
    return data.flat().map(elem => {
        return {
            h3: elem
        }
    })
}).then(groups => {
    groups.forEach(group => {
        d3.json(`h3/${group.h3}.json`).then(data => {
            return data.map(d => {
                return {
                    h3: d.h3,
                    freq: new Float32Array(d.freq),
                    type: d.type
                }
            })
        }).then(data => {
            onNewDataArrive(data)
        })
    })
})

