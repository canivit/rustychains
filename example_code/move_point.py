import json
import sys

point = json.loads(sys.stdin.readline())          
point['x'] += 7
point['y'] += 4
sys.stdout.write(json.dumps(point))
sys.stdout.write("\n")