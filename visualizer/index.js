const {DeckGL, H3HexagonLayer} = deck;

const deckgl = new DeckGL({
  mapStyle: 'https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json',
  initialViewState: {
    longitude: 12.6,
    latitude: 55.6,
    zoom: 10,
    minZoom: 5,
    maxZoom: 15,
    pitch: 40.5
  },
  controller: true
});

const data = d3.csv("data.csv")


const OPTIONS = ['time'];

const COLOR_RANGE = [
  [1, 152, 189],
  [73, 227, 206],
  [216, 254, 181],
  [254, 237, 177],
  [254, 173, 84],
  [209, 55, 78]
];

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
    if(key="time") {
      document.getElementById('time-label').innerHTML = `Day ${Math.floor(value/24) + 1}, ${(value%24) + 1}:00h`;
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
    getElevation: d => JSON.parse(d.freq)[options['time']],
    updateTriggers: {
      getElevation: d => JSON.parse(d.freq)[options['time']]
    }
  });

  deckgl.setProps({
    layers: [h3layer]
  });
}