"""Измерение ширин символов Arial в оригинале (Wine): S=папка_для_json FL=0|8 python3 tools/measure_arial.py 4 72 -> S/widths_FL_4_72.json; затем собрать tools/verify/arial_widths.json и запустить tools/gen_font_metrics.py."""
import subprocess,re,json,sys,os
S=os.environ['S']
res={}
sizes=list(range(int(sys.argv[1]),int(sys.argv[2])+1))
for i in range(0,len(sizes),5):
    chunk=sizes[i:i+5]
    subprocess.run(['python3','tools/measure_arial_probe.py',','.join(map(str,chunk)),S+'/tw.txt'],check=True)
    r=subprocess.run(['python3','tools/verify_original.py',S+'/tw.txt'],capture_output=True,text=True,env=dict(os.environ,VERIFY_TMP='/home/artem/.claude/jobs/f074b43d/tmp'))
    got={}
    for line in r.stdout.splitlines():
        m=re.match(r"\S+\s+out(\d+)([abcd])\s+.*оригинал='([^']*)'",line)
        if m: got[(int(m.group(1)),m.group(2))]=m.group(3)
    for h in chunk:
        parts=[got.get((h,c)) for c in 'abcd']
        if all(parts): res[h]=[int(x) for x in (''.join(parts)).strip(',').split(',')]
        else: print('missing',h,file=sys.stderr)
    json.dump(res,open(S+'/widths_%s%s_%s_%s.json'%(os.environ.get('FACE','').replace(' ',''),os.environ.get('FL','0'),sys.argv[1],sys.argv[2]),'w'))
    print(chunk,flush=True)
