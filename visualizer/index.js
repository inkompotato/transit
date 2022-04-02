const {DeckGL, H3HexagonLayer, MapController} = deck;

class MyMapController extends MapController {
  handleEvent(event) {
    if (event.type === "panmove") {
      let v = deckgl.viewManager._viewports[0]
      document.getElementById('debug-info').innerHTML = `${v.latitude.toFixed(2)}, ${v.longitude.toFixed(2)}`
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
    minZoom: 5,
    maxZoom: 15,
    pitch: 40.5,
  },
});

const days = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]

const data = d3.csv("../notebooks/data.csv").then(data => {
  return data.map(d => {
    return {
      h3: d.h3,
      freq: new Float32Array(JSON.parse(d.freq))
    }
  })
})

const OPTIONS = ['time'];

OPTIONS.forEach(key => {
  document.getElementById(key).oninput = renderLayer;
});

renderLayer();

function renderLayer () {
  const options = {};
  OPTIONS.forEach(key => {
    const value = +document.getElementById(key).value;
    document.getElementById(key + '-value').innerHTML = value;
    options[key] = parseInt(value);
    if(key === "time") {
      document.getElementById('time-label').innerHTML = `${days[Math.floor(value/24)]}, ${(value%24)}:00 - ${(value%24) + 1}:00`;
    }
  });

  const h3layer = new H3HexagonLayer({
    id: 'h3-hexagon-layer',
    data,
    pickable: false,
    wireframe: false,
    filled: true,
    extruded: true,
    elevationScale: 20,
    getHexagon: d => d.h3,
    getFillColor: d => [255, 128, 0],
    getElevation: d => d.freq[options['time']],
    updateTriggers: {
      getElevation: [options['time']]
    }
  });

  const testLayer = new H3HexagonLayer({
    id: 'h3-test-layer',
    data: [{h3: "831f05fffffffff"}],
    wireframe: false,
    filled: true,
    extruded: false,
    elevationScale: 0,
    getFillColor: d => [255, 255, 255],
    getHexagon: d => d.h3,
    getElevation: d => 12
  })

  deckgl.setProps({
    layers: [h3layer]
  });
}