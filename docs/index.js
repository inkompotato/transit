const {DeckGL, H3HexagonLayer, MapController, PathLayer} = deck;

class MyMapController extends MapController {
  handleEvent(event) {
    if (event.type === "panmove") {
      let v = deckgl.viewManager._viewports[0]
      let lat = v.latitude
      let lon = v.longitude
      let zoom = v.zoom

      document.getElementById('coordinate-info').innerHTML = `${lat.toFixed(2)}, ${lon.toFixed(2)} (${zoom})`
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

const h3_data = d3.csv("h3.csv").then(data => {
  return data.map(d => {
    return {
      h3: d.h3,
      freq: new Float32Array(JSON.parse(d.freq))
    }
  })
})

const route_data = d3.csv("routes.csv").then(data => {
  return data.map(d => {
    return {
      path: JSON.parse(d.geo),
      category: d.category
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
    data: h3_data,
    pickable: false,
    wireframe: false,
    filled: true,
    extruded: true,
    elevationScale: 20,
    getHexagon: d => d.h3,
    getFillColor: d => [255, 128, 0, 180],
    getElevation: d => d.freq[options['time']],
    updateTriggers: {
      getElevation: [options['time']]
    }
  });

  const route_layer = new PathLayer({
    id: 'route_layer',
    data: route_data,
    pickable: true,
    wireframe: false,
    widthMinPixels: 2,
    getPath: d => d.path,
    getColor: d => {
      switch (d.category) {
        case "0": return [17, 173, 125, 40]
        case "1": return [17, 54, 173, 40]
        case "2": return [250, 125, 0, 80]
        case "3": return [12, 250, 125, 0]
        default: return [12, 69, 250, 40]
      }

    },
    getWidth: d => 5
  })

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
    layers: [h3layer, route_layer]
  });
}