# Transit
Analyzing Public transit in Denmark and elsewhere. Using GTFS data and the Rejseplanen API.

## Build instructions
Uses pyO3 to build python modules in rust.

To build the python module, run `maturin develop`. If `maturin` is not available, run `pip install maturin`. Use a python kernel for the notebook.

## Useful links
### Rust crates
* https://github.com/georust/transitfeed
* https://docs.rs/petgraph/latest/petgraph/index.html 
* https://docs.rs/geojson/0.22.2/geojson/ 

## Rejseplanen API
add `format=json` to get the response in json instead of xml

### Location
The location service can be used to perform a pattern matching of a user input and to retrieve a list of possible matches in the journey planner database. Possible matches might be stops/stations, points of interest and addresses.

e.g.: `http://xmlopen.rejseplanen.dk/bin/rest.exe/location?input=<URL ENCODED INPUT>&format=json`

### StopsNearby
The stops nearby service will deliver all stops within a radius of a given coordinate.

e.g.: `http://xmlopen.rejseplanen.dk/bin/rest.exe/stopsNearby?coordX=12565796&coordY=55673063&maxRadius=1000&maxNumber=30`

### Trip
The trip service calculates a trip from a specified origin to a specified destination. These might be stop/station IDs or coordinates based on addresses and points of interest vali- dated by the location service or coordinates freely defined by the client.

The response will include real-time data.

#### Parameters
`originId`, `originCoordX`, `originCoordY`, `originCoordName`, `destId`, `destCoordX`, `destCoordY`, `destCoordName` - both origin and destination need to be clearly defined

Add a via stop or station with `viaId`

Departure time: `date`, `time`, if arrival use `searchForArrival=1`

Means of transportation: `useTog`, `useBus`, `useMetro`, default is 1 (true), set to 0 to exclude

`useBicycle=1` to only include means of transport that allow for bike carriage

further options:
```
maxWalkingDistanceDep=<distance in meter>
maxWalkingDistanceDest=<distance in meter>
maxCyclingDistanceDep=<distance in meter>
maxCyclingDistanceDest=<distance in meter>
```

e.g.: `http://xmlopen.rejseplanen.dk/bin/rest.exe/trip?originId=8600626&destId=8600637&format=json`

### DepartureBoard / ArrivalBoard
This method will return the next 20 departures (or less if not existing) from a given point in time.

The response will include real-time data.
Every departure will also contain a reference to the journey detail service.

e.g.: `http://xmlopen.rejseplanen.dk/bin/rest.exe/departureBoard?id=8600626&useBus=0&format=json`

### MultiDepartureBoard
The multi departure board is a combined departure board for up to 10 different stops. It can be retrieved by a service called multiDepartureBoard. This method will return the next 20 departures (or less if not existing) of the defined stops from a given point in time.

e.g.: `http://xmlopen.rejseplanen.dk/bin/rest.exe/multiDepartureBoard?id1=8600626&id2=5548&useTog=0`

### JourneyDetail
The journeyDetail service will deliver information about the complete route of a vehicle. This service can’t be called directly but only by reference URLs in a result of a trip or departureBoard request. It contains a list of all stops/stations of this journey including all departure and arrival times (**with realtime data if available**) and additional information like specific attributes about facilities and other texts.