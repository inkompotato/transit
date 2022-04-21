from esy.osmfilter import osm_colors as CC
from esy.osmfilter import run_filter 
from esy.osmfilter import Node, Way, Relation

from esy.osmfilter import export_geojson


import os, sys


PBF_inputfile = os.path.join(os.getcwd(),
                             'notebooks/denmark-latest.osm.pbf')

JSON_outputfile = os.path.join(os.getcwd(),
'denmark_all.json')

pf = {Node: {}, Way: {"highway":["footway", 'path', 'pedestrian', 'cycleway', 'residential','unclassified', 'tertiary', 'secondary']}, Relation: {}}

wf = [((),())]

bf = [('foot', 'no')]


[Data,Elements] = run_filter('dk_walk',
            PBF_inputfile, 
            JSON_outputfile, 
            prefilter = pf,
            whitefilter = wf, 
            blackfilter = bf, 
            NewPreFilterData=True, 
            CreateElements=True, 
            LoadElements=False,
            verbose=True)



export_geojson(Data['Way'], Data, filename='dkall.geojson',jsontype='Line')