from esy.osmfilter import osm_colors as CC
from esy.osmfilter import run_filter 
from esy.osmfilter import Node, Way, Relation

from esy.osmfilter import export_geojson


import os

def main():
        
    PBF_inputfile = os.path.join(os.path.dirname(__file__),
                                'denmark-latest.osm.pbf')

    JSON_outputfile = os.path.join(os.path.dirname(__file__), 'data','denmark_all.json')

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



    export_geojson(Data['Way'], Data, filename= os.path.join(os.path.dirname(__file__), 'data', 'dkall.geojson'), jsontype='Line')

if __name__ == "__main__":
    main()