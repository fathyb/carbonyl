//! Glyph lookup tables, indexed by a mask of filled sub-cells.
//! Bit `row * 2 + col` is set when that sub-cell uses the foreground color.
//! Generated from the Unicode names of the Symbols for Legacy Computing blocks.

pub const SEXTANTS: [&str; 64] = [
    "\u{20}", "\u{1FB00}", "\u{1FB01}", "\u{1FB02}", "\u{1FB03}", "\u{1FB04}", "\u{1FB05}", "\u{1FB06}",
    "\u{1FB07}", "\u{1FB08}", "\u{1FB09}", "\u{1FB0A}", "\u{1FB0B}", "\u{1FB0C}", "\u{1FB0D}", "\u{1FB0E}",
    "\u{1FB0F}", "\u{1FB10}", "\u{1FB11}", "\u{1FB12}", "\u{1FB13}", "\u{258C}", "\u{1FB14}", "\u{1FB15}",
    "\u{1FB16}", "\u{1FB17}", "\u{1FB18}", "\u{1FB19}", "\u{1FB1A}", "\u{1FB1B}", "\u{1FB1C}", "\u{1FB1D}",
    "\u{1FB1E}", "\u{1FB1F}", "\u{1FB20}", "\u{1FB21}", "\u{1FB22}", "\u{1FB23}", "\u{1FB24}", "\u{1FB25}",
    "\u{1FB26}", "\u{1FB27}", "\u{2590}", "\u{1FB28}", "\u{1FB29}", "\u{1FB2A}", "\u{1FB2B}", "\u{1FB2C}",
    "\u{1FB2D}", "\u{1FB2E}", "\u{1FB2F}", "\u{1FB30}", "\u{1FB31}", "\u{1FB32}", "\u{1FB33}", "\u{1FB34}",
    "\u{1FB35}", "\u{1FB36}", "\u{1FB37}", "\u{1FB38}", "\u{1FB39}", "\u{1FB3A}", "\u{1FB3B}", "\u{2588}",
];

pub const OCTANTS: [&str; 256] = [
    "\u{20}", "\u{1CEA8}", "\u{1CEAB}", "\u{1FB82}", "\u{1CD00}", "\u{2598}", "\u{1CD01}", "\u{1CD02}",
    "\u{1CD03}", "\u{1CD04}", "\u{259D}", "\u{1CD05}", "\u{1CD06}", "\u{1CD07}", "\u{1CD08}", "\u{2580}",
    "\u{1CD09}", "\u{1CD0A}", "\u{1CD0B}", "\u{1CD0C}", "\u{1FBE6}", "\u{1CD0D}", "\u{1CD0E}", "\u{1CD0F}",
    "\u{1CD10}", "\u{1CD11}", "\u{1CD12}", "\u{1CD13}", "\u{1CD14}", "\u{1CD15}", "\u{1CD16}", "\u{1CD17}",
    "\u{1CD18}", "\u{1CD19}", "\u{1CD1A}", "\u{1CD1B}", "\u{1CD1C}", "\u{1CD1D}", "\u{1CD1E}", "\u{1CD1F}",
    "\u{1FBE7}", "\u{1CD20}", "\u{1CD21}", "\u{1CD22}", "\u{1CD23}", "\u{1CD24}", "\u{1CD25}", "\u{1CD26}",
    "\u{1CD27}", "\u{1CD28}", "\u{1CD29}", "\u{1CD2A}", "\u{1CD2B}", "\u{1CD2C}", "\u{1CD2D}", "\u{1CD2E}",
    "\u{1CD2F}", "\u{1CD30}", "\u{1CD31}", "\u{1CD32}", "\u{1CD33}", "\u{1CD34}", "\u{1CD35}", "\u{1FB85}",
    "\u{1CEA3}", "\u{1CD36}", "\u{1CD37}", "\u{1CD38}", "\u{1CD39}", "\u{1CD3A}", "\u{1CD3B}", "\u{1CD3C}",
    "\u{1CD3D}", "\u{1CD3E}", "\u{1CD3F}", "\u{1CD40}", "\u{1CD41}", "\u{1CD42}", "\u{1CD43}", "\u{1CD44}",
    "\u{2596}", "\u{1CD45}", "\u{1CD46}", "\u{1CD47}", "\u{1CD48}", "\u{258C}", "\u{1CD49}", "\u{1CD4A}",
    "\u{1CD4B}", "\u{1CD4C}", "\u{259E}", "\u{1CD4D}", "\u{1CD4E}", "\u{1CD4F}", "\u{1CD50}", "\u{259B}",
    "\u{1CD51}", "\u{1CD52}", "\u{1CD53}", "\u{1CD54}", "\u{1CD55}", "\u{1CD56}", "\u{1CD57}", "\u{1CD58}",
    "\u{1CD59}", "\u{1CD5A}", "\u{1CD5B}", "\u{1CD5C}", "\u{1CD5D}", "\u{1CD5E}", "\u{1CD5F}", "\u{1CD60}",
    "\u{1CD61}", "\u{1CD62}", "\u{1CD63}", "\u{1CD64}", "\u{1CD65}", "\u{1CD66}", "\u{1CD67}", "\u{1CD68}",
    "\u{1CD69}", "\u{1CD6A}", "\u{1CD6B}", "\u{1CD6C}", "\u{1CD6D}", "\u{1CD6E}", "\u{1CD6F}", "\u{1CD70}",
    "\u{1CEA0}", "\u{1CD71}", "\u{1CD72}", "\u{1CD73}", "\u{1CD74}", "\u{1CD75}", "\u{1CD76}", "\u{1CD77}",
    "\u{1CD78}", "\u{1CD79}", "\u{1CD7A}", "\u{1CD7B}", "\u{1CD7C}", "\u{1CD7D}", "\u{1CD7E}", "\u{1CD7F}",
    "\u{1CD80}", "\u{1CD81}", "\u{1CD82}", "\u{1CD83}", "\u{1CD84}", "\u{1CD85}", "\u{1CD86}", "\u{1CD87}",
    "\u{1CD88}", "\u{1CD89}", "\u{1CD8A}", "\u{1CD8B}", "\u{1CD8C}", "\u{1CD8D}", "\u{1CD8E}", "\u{1CD8F}",
    "\u{2597}", "\u{1CD90}", "\u{1CD91}", "\u{1CD92}", "\u{1CD93}", "\u{259A}", "\u{1CD94}", "\u{1CD95}",
    "\u{1CD96}", "\u{1CD97}", "\u{2590}", "\u{1CD98}", "\u{1CD99}", "\u{1CD9A}", "\u{1CD9B}", "\u{259C}",
    "\u{1CD9C}", "\u{1CD9D}", "\u{1CD9E}", "\u{1CD9F}", "\u{1CDA0}", "\u{1CDA1}", "\u{1CDA2}", "\u{1CDA3}",
    "\u{1CDA4}", "\u{1CDA5}", "\u{1CDA6}", "\u{1CDA7}", "\u{1CDA8}", "\u{1CDA9}", "\u{1CDAA}", "\u{1CDAB}",
    "\u{2582}", "\u{1CDAC}", "\u{1CDAD}", "\u{1CDAE}", "\u{1CDAF}", "\u{1CDB0}", "\u{1CDB1}", "\u{1CDB2}",
    "\u{1CDB3}", "\u{1CDB4}", "\u{1CDB5}", "\u{1CDB6}", "\u{1CDB7}", "\u{1CDB8}", "\u{1CDB9}", "\u{1CDBA}",
    "\u{1CDBB}", "\u{1CDBC}", "\u{1CDBD}", "\u{1CDBE}", "\u{1CDBF}", "\u{1CDC0}", "\u{1CDC1}", "\u{1CDC2}",
    "\u{1CDC3}", "\u{1CDC4}", "\u{1CDC5}", "\u{1CDC6}", "\u{1CDC7}", "\u{1CDC8}", "\u{1CDC9}", "\u{1CDCA}",
    "\u{1CDCB}", "\u{1CDCC}", "\u{1CDCD}", "\u{1CDCE}", "\u{1CDCF}", "\u{1CDD0}", "\u{1CDD1}", "\u{1CDD2}",
    "\u{1CDD3}", "\u{1CDD4}", "\u{1CDD5}", "\u{1CDD6}", "\u{1CDD7}", "\u{1CDD8}", "\u{1CDD9}", "\u{1CDDA}",
    "\u{2584}", "\u{1CDDB}", "\u{1CDDC}", "\u{1CDDD}", "\u{1CDDE}", "\u{2599}", "\u{1CDDF}", "\u{1CDE0}",
    "\u{1CDE1}", "\u{1CDE2}", "\u{259F}", "\u{1CDE3}", "\u{2586}", "\u{1CDE4}", "\u{1CDE5}", "\u{2588}",
];
