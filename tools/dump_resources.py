# Разбор ресурсов Win32: меню (RT_MENU=4), диалоги (RT_DIALOG=5), строки (RT_STRING=6).
import sys, struct, pefile, json
def rd_sz(b, o):
    s=''
    while True:
        c=struct.unpack_from('<H',b,o)[0]; o+=2
        if c==0: return s,o
        s+=chr(c)
def menu(b):
    ver,hdr=struct.unpack_from('<HH',b,0); o=4+hdr
    if ver!=0: return ['<menuex>']
    def items(o, depth):
        out=[]
        while True:
            fl=struct.unpack_from('<H',b,o)[0]; o+=2
            if fl&0x10:  # popup
                t,o=rd_sz(b,o); sub,o=items(o,depth+1); out.append(('  '*depth)+'▸ '+t); out+=sub
            else:
                idn=struct.unpack_from('<H',b,o)[0]; o+=2; t,o=rd_sz(b,o)
                out.append(('  '*depth)+(t if t else '———')+(f'  [{idn}]' if t else ''))
            if fl&0x80: return out,o
    return items(o,0)[0]
CLS={0x80:'BUTTON',0x81:'EDIT',0x82:'STATIC',0x83:'LISTBOX',0x84:'SCROLLBAR',0x85:'COMBOBOX'}
def dialog(b):
    o=0
    sig=struct.unpack_from('<HH',b,0)
    ex = sig==(1,0xffff)
    if ex:
        _,_,helpid,exst,st,n,x,y,w,h=struct.unpack_from('<HHIIIHhhhh',b,0); o=26
    else:
        st,exst,n,x,y,w,h=struct.unpack_from('<IIHhhhh',b,0); o=18
    def sz_or_ord(o):
        v=struct.unpack_from('<H',b,o)[0]
        if v==0: return '',o+2
        if v==0xffff: return '#%d'%struct.unpack_from('<H',b,o+2)[0],o+4
        return rd_sz(b,o)
    _,o=sz_or_ord(o); cls,o=sz_or_ord(o); title,o=rd_sz(b,o)
    if st&0x40:
        o+=4 if not ex else 6
        font,o=rd_sz(b,o)
    res={'title':title,'size':[w,h],'controls':[]}
    for i in range(n):
        o=(o+3)&~3
        if ex:
            hid,cexst,cst,cx,cy,cw,ch,cid=struct.unpack_from('<IIIhhhhI',b,o); o=24
            o=((o+3)&~3); 
        else:
            cst,cexst,cx,cy,cw,ch,cid=struct.unpack_from('<IIhhhhH',b,o); o+=18
        ccls,o=sz_or_ord(o); ct,o=sz_or_ord(o)
        ex_n=struct.unpack_from('<H',b,o)[0]; o+=2+ex_n
        if ccls.startswith('#'): ccls=CLS.get(int(ccls[1:]),ccls)
        if ccls=='BUTTON':
            t=cst&0xF; ccls={0:'BUTTON',1:'DEFBUTTON',2:'CHECK',3:'AUTOCHECK',4:'RADIO',5:'RADIO3',7:'GROUP',9:'AUTORADIO'}.get(t,'BUTTON')
        if not (cst&0x10000000): ccls+='(hidden)'
        res['controls'].append([ccls,ct,cid,[cx,cy,cw,ch]])
    return res
def strings(b, base):
    o=0; out=[]
    for i in range(16):
        n=struct.unpack_from('<H',b,o)[0]; o+=2
        s=b[o:o+2*n].decode('utf-16le'); o+=2*n
        if s: out.append((base*16+i, s))
    return out
pe=pefile.PE(sys.argv[1])
menus={}; dialogs={}; strs=[]
for t in pe.DIRECTORY_ENTRY_RESOURCE.entries:
    for e in t.directory.entries:
        name = e.name.__str__() if e.name else e.id
        for l in e.directory.entries:
            off=l.data.struct.OffsetToData; size=l.data.struct.Size
            b=pe.get_data(off,size)
            try:
                if t.id==4: menus[name]=menu(b)
                elif t.id==5: dialogs[name]=dialog(b)
                elif t.id==6: strs+=strings(b,e.id-1)
            except Exception as ex: print('err',t.id,name,ex,file=sys.stderr)
json.dump({'menus':menus,'dialogs':dialogs,'strings':strs},open(sys.argv[2],'w'),ensure_ascii=False,indent=1)
print(len(menus),'menus',len(dialogs),'dialogs',len(strs),'strings')
