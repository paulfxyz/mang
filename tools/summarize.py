import json,sys,glob,collections
rows=[]
for f in sorted(glob.glob(sys.argv[1])): rows+=json.load(open(f))
by=collections.defaultdict(list)
for r in rows: by[r['model']].append(r)
print(f"{'model':32} n  err empty json strict pass/chk  avg_s  tot$     danger_nocomment")
for m,rs in by.items():
    n=len(rs); err=sum('error' in r for r in rs); ok=[r for r in rs if 'error' not in r]
    empty=sum(r['empty_content'] for r in ok); js=sum(r['json_ok'] for r in ok); st=sum(r['strict'] for r in ok)
    chk=[r for r in ok if r['pass'] is not None]; ps=sum(bool(r['pass']) for r in chk)
    lat=sum(r['latency'] for r in ok)/max(1,len(ok)); cost=sum((r['cost'] or 0) for r in ok)
    dn=sum(1 for r in ok if r['danger_in_cmds'] and not r['has_safety_comment'])
    print(f"{m:32} {n:2} {err:3} {empty:5} {js:4} {st:6} {ps}/{len(chk):<7} {lat:5.2f}  {cost:.5f}  {dn}")
if len(sys.argv)>2:
    for r in rows:
        print(r['model'].split('/')[1][:14],r['case'],'|',r.get('finish'),r.get('reason_tok'),'|',r.get('cmds'),'|',(r.get('error') or r.get('raw') or '')[:100])
