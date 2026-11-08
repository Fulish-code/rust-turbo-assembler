;**********************************************************
; CADENAS.ASM
;
; Este programa prueba los procedimientos para cadenas del
; capítulo 7: astrlen, astrupr y aatoi (con sus auxiliares
; obtenSigno, obtenBase, atou y valC).
;
;** El programa principal no viene en el manual: se agregó para
;** poder ensamblar, ejecutar y depurar (TD) los procedimientos.
;
; Como astrlen usa scasb, que trabaja con ES:DI, el programa
; inicializa ES igual que DS.
;
; El pseudocódigo de este programa es:
;
;   longi = astrlen(texto)
;   astrupr(texto)
;   num1 = aatoi(snum1)
;   num2 = aatoi(snum2)
;   num3 = aatoi(snum3)
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
texto   db      'hola mundo', 0
snum1   db      '-1Ah', 0               ; -26 en hexadecimal
snum2   db      '101b', 0               ; 5 en binario
snum3   db      '+250', 0               ; 250 en decimal
longi   dw      ?
num1    dw      ?
num2    dw      ?
num3    dw      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos y
        mov     es, ax                  ; el segmento extra

        mov     si, offset texto        ; longi = astrlen(texto)
        call    astrlen
        mov     [longi], cx

        call    astrupr                 ; astrupr(texto)

        mov     si, offset snum1        ; num1 = aatoi(snum1)
        call    aatoi
        mov     [num1], ax

        mov     si, offset snum2        ; num2 = aatoi(snum2)
        call    aatoi
        mov     [num2], ax

        mov     si, offset snum3        ; num3 = aatoi(snum3)
        call    aatoi
        mov     [num3], ax

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** PROCEDIMIENTOS ************************************

;**********************************************************
; ASTRLEN
;
; Este procedimiento regresa la longitud de una cadena
; terminada en 0.
;
; Parámetros:
;
;         SI = cadena
;
; Regresa:
;
;         CX = strlen(cadena)
;
;
; El pseudocódigo de este procedimiento es:
;
;   int astrlen(char *cadena)
;   {
;      p = cadena

;
;      while(*p++);
;
;      return p - cadena - 1
;    }
;**********************************************************
proc    astrlen
        push    ax                      ; Preserva AX, DI
        push    di

        mov     di, si                  ; DI = SI
        xor     al, al                  ; AL = 0

        cld                             ; Autoincrementa DI
@@whi:  scasb                           ; while([DI++]);
        jnz     @@whi

        mov     cx, di                  ; CX = DI - SI - 1
        sub     cx, si
        dec     cx

;** CORREGIDO: el manual dice popr di; la instrucción es pop.
        pop     di                      ; Restaura DI, AX
        pop     ax
        ret
endp    astrlen

;**********************************************************
; ASTRUPR
;
; Este procedimiento convierte una cadena terminada en 0 a
; mayúsculas.
;
; Parámetros:
;
;         SI = cadena
;
; Regresa:
;
;         SI = scadena
;
; El pseudocódigo de este procedimiento es:
;
;
;   char *astrupr(char *cadena)
;   {
;      n = astrlen(cadena)
;      if(!n) goto @@fin
;
;      p = q = cadena

;      do
;      {
;          ch = *(p++)
;          if(ch >= 'a' && ch <= 'z') toupper(ch)
;          *(q++) = ch
;      }
;      while(--n > 0)
;
;      return cadena
;    }
;**********************************************************
proc    astrupr
        push    ax                      ; Preserva AX, CX, SI, DI
        push    cx
        push    si
        push    di

        call    astrlen                 ; CX = strlen(cadena)

        jcxz    @@fin                   ; if(!CX) goto @@fin

        mov     di, si                  ; DI = SI

        cld                             ; Autoincrementa SI, DI
@@do:                                   ; do
                                        ; {
        lodsb                           ;    AL = [SI++]
        cmp     al, 'a'                 ;    if(AL < 'a' ||
        jb      @@sig                   ;       AL > 'z')
        cmp     al, 'z'
        ja      @@sig
        sub     al, 'a'-'A'             ;       AL = toupper(AL)

@@sig:  stosb                           ;    [DI++] = AL
        loop    @@do                    ; }
                                        ; while(--CX > 0)
@@fin:  pop     di                      ; Restaura DI, SI, CX, AX
        pop     si
        pop     cx
        pop     ax
        ret
endp    astrupr

;**********************************************************
; AATOI
;
; Este procedimiento convierte una cadena terminada en 0,
; que representa un número de tipo palabra con signo a

; binario. El signo si existe debe ser el primer carácter
; de la cadena. La cadena puede terminar en d, b, h
; indicando la base del número. No se permiten espacios en
; la cadena.
;
; Parámetros:
;
;          SI = cadena con el número
;
; Regresa:
;
;          AX = número en binario
;
; El pseudocódigo del procedimiento es:
;
;     int aatoi(char *s)
;     {
;          astrup(cadena)
;          l = astrlen(s)
;          signo = obtenSigno(&s, &l)
;          base = obtenBase(s, &l)
;          n = atou(s, base, l)
;          if(signo) n *= -1
;          return n
;       }
;**********************************************************
proc    aatoi
        push    bx                      ; Preserva BX, CX, DX, SI
        push    cx
        push    dx
        push    si

        call    astrupr                 ; strup(cadena)
        call    astrlen                 ; CX = strlen(cadena)
        call    obtenSigno              ; DX = [SI] == '-',
                                        ; SI++, CX--
        call    obtenBase               ; BX = base, CX--
        call    atou                    ; AX = atou(cadena)
        cmp     dx, 0                   ; if(dx == 0)
        je      @@sig                   ;    goto @@sig
        neg     ax                      ; ax = -ax

@@sig:  pop     si                      ; Restaura SI, DX, CX, BX
        pop     dx
        pop     cx
        pop     bx
        ret
endp    aatoi

;**********************************************************
; OBTENSIGNO
;
; Este procedimiento que solo debe llamarlo aatoi, lee el
; primer carácter de una cadena que representa un número de
; tipo palabra con signo y determina si este carácter
; representa el signo del número.
;
; Parámetros:
;
;        SI = cadena con el número
;        CX = Longitud de la cadena
;
; Regresa:
;
;        CX : if([si] == '+' || [si] == '-') CX--
;        DX = [si] == '-'
;        SI : if([si] == '+' || [si] == '-') SI++
;
; El pseudocódigo de este procedimiento es:
;
;    int obtenSigno(char **s, int *l)
;    {
;       signo = 0
;       if(**s == '+') goto @@pos
;       if(**s == '-') goto @@neg
;       goto @@fin
;
;    @@neg:
;       signo = 1
;    @@pos:
;       (*s)++
;       *l--
;
;       return signo
;    }
;**********************************************************
proc    obtenSigno
        xor     dx, dx                  ; dx = 0
        cmp     [byte si], '+'          ; if([si] == '+')
        je      @@pos                   ;    goto @@pos
        cmp     [byte si], '-'          ; if([si] == '-')
        je      @@neg                   ;    goto @@neg
        jmp     @@fin                   ; goto @@fin

@@neg:  mov     dx, 1                   ; Dx = 1
@@pos:  inc     si                      ; SI++
        dec     cx                      ; CX--
@@fin:  ret
endp    obtenSigno

;**********************************************************
; OBTENBASE
;
; Este procedimiento que solo debe llamarlo aatoi, lee el
; último carácter de una cadena que representa un número de
; tipo palabra sin signo y determina la base en que está
; representada el número. Por omisión la base es 10.
;
; Parámetros:
;
;        SI = cadena con el número
;        CX = Longitud de la cadena
;
; Regresa:
;
;        BX : if([si+cx-1] == 'B') BX = 2
;             else if([si+cx-1] == 'H') BX = 16
;             else BX = 10
;        CX : if([si+cx-1] == 'B' || [si+cx-1] == 'H' ||
;                 [si+cx-1] == 'D') CX--
;
; El pseudocódigo de este procedimiento es:
;
;    int obtenBase(char *s, int *l)
;    {
;       p = s + astrlen(p) -1
;       base = 10
;       if(*p == 'B') base = goto @@bin
;       if(*p == 'H') base = goto @@hex
;       if(*p == 'D') base = goto @@dec
;       goto @@fin
;
;    @@bin:
;       base = 2
;       goto @@dec
;    @@hex:
;       base = 16
;    @@dec:
;       *l--
;    @@fin:
;       return base
;    }
;**********************************************************
proc    obtenBase
        push    si                      ; Preserva SI
        add     si, cx                  ; SI = cadena + strlen(
        dec     si                      ;      cadena) - 1

        mov     bx, 10                  ; base = 10

        cmp     [byte si], 'B'          ; if([si] == 'B')
        je      @@bin                   ;    goto @@bin

        cmp     [byte si], 'H'          ; if([si] == 'H')
        je      @@hex                   ;    goto @@hex
        cmp     [byte si], 'D'          ; if([si] == 'D')
        je      @@dec                   ;    goto @@dec
        jmp     @@fin                   ; goto @@fin

@@bin:  mov     bx, 2                   ; base = 2
        jmp     @@dec                   ; goto @@dec
@@hex:  mov     bx,16                   ; Base = 16
@@dec:  dec     cx                      ; CX--
@@fin:
        pop     si                      ; Restaura SI
        ret
endp    obtenBase

;**********************************************************
; ATOU
;
; Este procedimiento que solo debe llamarlo aatoi,
; convierte una cadena que representa un número de tipo
; palabra sin signo a binario.
;
; Parámetros:
;
;         SI = cadena con el número
;         BX = 2, 10, 16, base del número
;         CX = strlen(cadena)
;
; Regresa:
;
;          AX = número en binario
;
; El pseudocódigo de este procedimiento es:
;
;    int atou (char *s, int base, int l)
;    {
;       n = 0
;       if(!l) goto @@fin
;
;       do
;       {
;           n = base*n + valC(*s)
;           s++
;       }
;       while(--l > 0)
;       return n
;    }
;**********************************************************
proc    atou
        push    dx                      ; Preserva DX, DI
        push    di

        xor     ax, ax                  ; n = 0

        jcxz    @@fin                   ; if(!CX) goto @@fin

        xor     di, di                  ; n = 0
@@do:                                   ; do
                                        ; {
        mov     ax, di                  ;    AX = base*n
        mul     bx
        mov     dl, [byte si]           ;    DX = [SI]
        xor     dh, dh
        call    valC                    ;    DX = val([SI])
        add     ax, dx                  ;    AX = base*n + DX
        mov     di, ax                  ;    n = AX
        inc     si                      ;    SI++
        loop    @@do                    ; }
                                        ; while(--CX > 0)

        mov     ax, di
@@fin:  pop     di                      ; Restaura DI, DX
        pop     dx
        ret
endp    atou

;**********************************************************
; VALC
;
; Este procedimiento que solo debe llamarlo atou, convierte
; un carácter que representa un número a su valor en
; binario.
;
; Parámetros:
;
;        DX = carácter
;
; Regresa:
;
;        DX = número
;
; El pseudocódigo de este procedimiento es:
;
;   int valC(char ch)
;   {
;      if(ch > '9') goto @@hex
;      return ch - '0'
;
;   @@hex:
;      return ch - ('A'- 10)
;   }
;**********************************************************

proc    valC
        cmp     dx, '9'
        ja      @@hex

        sub     dx, '0'
        ret

@@hex:  sub     dx, 'A' - 10
        ret
endp    valC

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
